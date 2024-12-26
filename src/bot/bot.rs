use std::vec;

use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};
use crate::{crawl::crawl::LolcheggCrawler, db::db::Storage};
use std::sync::{Arc, RwLock};

use super::traits::{self, Mode};

// todo. clone 대신 Arc로 wrapping 하는 것 고려
#[derive(Clone)]
pub struct LolcheBot {
    pub loader: LolcheggCrawler,
    pub stg: Storage,
}
// pub struct LolcheBot <L, S, E> 
//     where 
//         L: traits::DeckLoader<E>,
//         S: traits::Storage<E>,
//         E : std::error::Error 
// {
//     loader: L,
//     stg : S,
// }

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Challenge,
    Rollback,
}

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text.
    Help,
    #[command(description = "show current mode")]
    Mode,
    #[command(description = "switch the mode")]
    Switch,
    #[command(description = "bring updated decks")]
    Update,
    #[command(description = "delete records")]
    Reset,
    #[command(description = "show completed decks")]
    Done,
    #[command(description = "fix path to decks")]
    Fix
}

impl LolcheBot {

    pub async fn run(self, token: &str) {
        let bot = Bot::new(token);
        let shared_lolchebot = Arc::new(RwLock::new(self));

        Dispatcher::builder(
            bot,
            schema()
        )
        .dependencies(dptree::deps![InMemStorage::<State>::new(), shared_lolchebot])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    }
}



fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Mode].endpoint(mode))
        .branch(case![Command::Switch].endpoint(switch))
        .branch(case![Command::Update].endpoint(update))
        .branch(case![Command::Reset].endpoint(reset))
        .branch(case![Command::Done].endpoint(done))
        .branch(case![Command::Fix].endpoint(fix))
        .branch(dptree::endpoint(invalid_state))

        ;

    let message_handler = Update::filter_message()
        .branch(command_handler);

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::Challenge].endpoint(success))
        .branch(case![State::Rollback].endpoint(rollback))
    ;

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
        
}

/*
Todo
Error handling 함수
*/
// async fn error_handler<H, Fut> (
//     handler: H,
//     bot: Bot,
//     msg: Message,
// ) -> HandlerResult
// where
//     H: Fn(Bot, Message) -> Fut + Send + Sync + 'static,
//     Fut: std::future::Future<Output = HandlerResult> + Send,
// {
//     match handler(bot, msg).await {
//         Ok(_) => Ok(()),
//         Err(e) => {
//             .await;
//             Ok(()) // Prevents the error from propagating further.,\
//         },
//     }
// }


async fn help(bot: Bot,  msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn mode(bot: Bot, lolche_bot:Arc<RwLock<LolcheBot>>, msg: Message) -> HandlerResult {
    
    let mode = lolche_bot.read().unwrap().stg.select_mode()?;

    bot.send_message(msg.chat.id, format!("현재 모드 : {}", mode.msg())).await?;
    
    Ok(())
}

async fn switch(bot: Bot, dialogue: MyDialogue, lolche_bot:Arc<RwLock<LolcheBot>>, msg: Message) -> HandlerResult {
    // todo. DB에서 현재 모드 변경
    let mode = lolche_bot.read().unwrap().stg.select_mode()?.switch();
    
    lolche_bot.read().unwrap().stg.upsert_mode(&mode)?;
    
    bot.send_message(msg.chat.id, format!("모드 변경 성공. 현재 모드 : {}", mode.msg())).await?;
    
    Ok(())
}

async fn update(bot: Bot, msg: Message, dialogue: MyDialogue, lolche_bot:Arc<RwLock<LolcheBot>>) -> HandlerResult {
    
    let mode = lolche_bot.read().unwrap().stg.select_mode()?;

    let done = lolche_bot.read().unwrap().stg.retrieve_done(&mode)?;

    // todo 이렇게 옮기는거 말고 copy 해서 넘길 순 없나??
    let deck_result = tokio::task::spawn_blocking(move || {
        lolche_bot.read().unwrap().loader.recommended_deck(&mode)
    })
    .await?;

    if deck_result.is_err() {
        bot.send_message(msg.chat.id, format!("오류 발생. {:?}", deck_result)).await?;
        return Ok(());
    }

    let updated_deck = deck_result.unwrap();
    let [normal, special] = todo_deck(updated_deck, done);

    log::info!("{:?}", normal);
    log::info!("{:?}", special);

    bot.send_message(msg.chat.id, "다음 일반 덱")
    .reply_markup(InlineKeyboardMarkup::new(
        normal.into_iter().map(|s| vec![InlineKeyboardButton::callback(s.clone(), s)]).collect::<Vec<Vec<InlineKeyboardButton>>>()
        ))
    .await?;

    bot.send_message(msg.chat.id, "잔여 특수 덱")
    .reply_markup(InlineKeyboardMarkup::new(
        special.into_iter().map(|s| vec![InlineKeyboardButton::callback(s.clone(), s)]).collect::<Vec<Vec<InlineKeyboardButton>>>()
        ))
    .await?;

    dialogue.update(State::Challenge).await?;
    Ok(())
}

async fn reset(bot: Bot, msg: Message, lolche_bot:Arc<RwLock<LolcheBot>>) -> HandlerResult {
    
    let mode = lolche_bot.read().unwrap().stg.select_mode()?;

    lolche_bot.read().unwrap().stg.delete_all(&mode)?;

    bot.send_message(msg.chat.id, format!("모드 {}에 대한 이력 삭제 완료", mode.msg())).await?;
    Ok(())
}

// memo. iter-map 안에서는 비동기를 날리지 못 함
async fn done(bot: Bot, dialogue: MyDialogue, msg: Message, lolche_bot:Arc<RwLock<LolcheBot>>) -> HandlerResult {
    
    let mode = lolche_bot.read().unwrap().stg.select_mode()?;

    let done = lolche_bot.read().unwrap().stg.retrieve_done(&mode)?;
    // 버튼 보내기
    bot.send_message(msg.chat.id, "완료 내역")
       .reply_markup(
            InlineKeyboardMarkup::new(
                done.iter()
                    .map(|s| vec![InlineKeyboardButton::callback(s.clone(), s)])
                    .collect::<Vec<Vec<InlineKeyboardButton>>>()
        ))
       .await?;
    dialogue.update(State::Rollback).await?;
    Ok(())
}

async fn fix(bot: Bot, msg: Message, lolche_bot:Arc<RwLock<LolcheBot>>) -> HandlerResult {
    
    tokio::task::spawn_blocking(move || {
        let mut mutable_bot = lolche_bot.write().unwrap();
        mutable_bot.loader.update_css_path();
    })
    .await?;
    
    bot.send_message(msg.chat.id, "css path 수정 완료").await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "잘못된 커맨드").await?;
    Ok(())
}

async fn success(bot: Bot, 
                dialogue: MyDialogue,
                q: CallbackQuery, 
                lolche_bot:Arc<RwLock<LolcheBot>>) -> HandlerResult 
{
    if let Some(deck) = &q.data {
        
        let mode = lolche_bot.read().unwrap().stg.select_mode()?; 

        lolche_bot.read().unwrap().stg.record_done(deck, &mode)?;

        bot.send_message( dialogue.chat_id(), format!("{} 완료!", deck)).await?;
        dialogue.exit().await?;
    }
    Ok(())
}

async fn rollback(bot: Bot, 
                dialogue: MyDialogue,
                q: CallbackQuery, 
                lolche_bot:Arc<RwLock<LolcheBot>>) -> HandlerResult 
{
    if let Some(deck) = &q.data {
        let mode = lolche_bot.read().unwrap().stg.select_mode()?; 
        lolche_bot.read().unwrap().stg.delete_record(&mode, deck)?;
        bot.send_message(dialogue.chat_id(), format!("{} 롤백 완료", deck)).await?;
        dialogue.exit().await?;
    }
    Ok(())
}

async fn error_handle(bot: Bot,  dialogue: MyDialogue, result: HandlerResult) ->HandlerResult{

    match result {
        Ok(_) => {},
        Err(error) => {bot.send_message(dialogue.chat_id(), format!("오류 발생 : {}", error )).await?;},
    }
    Ok(())
}

fn todo_deck (mut recom : Vec<String> , done : Vec<String>) -> [Vec<String>;2] {
    use std::collections::HashMap;

    let mut done_map: HashMap<String, bool> = HashMap::new();
    // todo. String 말고 reference?
    for d in done {
        done_map.insert(d, true);
    }

    let mut normal = Vec::<String>::new();
    let mut special = Vec::<String>::new();
    let mut is_normal_picked = false;

    for i in (0..recom.len()).rev() {

        let target = recom.remove(i);

        if let Some(true) =done_map.get(&target) {
            continue;
        } 

        if target.starts_with("[") {
            special.push(target);
        } else if !is_normal_picked {
            normal.push(target);
            is_normal_picked = true;
        }
    }

    [normal,special]
}

#[cfg(test)]
mod test {
    use super::*;

    const TOKEN: &str = "";
    const CHAT_ID : i64 = 1;

    #[tokio::test]
    async fn sample_send_test() {
        let bot = Bot::new(TOKEN);
        match bot.send_message(ChatId(CHAT_ID), "sample message").await {
            Ok(_) => print!("Success"),
            Err(_) => print!("Fail")
        }
    }

    #[tokio::test]
    async fn run_test() {
        pretty_env_logger::init();
        log::info!("Test Start");
        let my_bot = LolcheBot{
            loader : LolcheggCrawler::new(),
            stg: Storage::new(),
        };

        my_bot.run(TOKEN).await;
    }
}

