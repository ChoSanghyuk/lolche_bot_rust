use std::vec;

use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};
use tokio::runtime::Runtime;

// pub type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(Clone)]
 pub struct LolcheBot {
    test: &'static str
}

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

    pub fn new() -> Self {
        Self{
            test: "hello~"
        }
    }
}

pub async fn run(token: &str) {
    let bot = Bot::new(token);
    Dispatcher::builder(
        bot,
        schema()
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new(), LolcheBot::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
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
        .branch(dptree::endpoint(invalid_state));
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



async fn help(bot: Bot,  msg: Message, info:LolcheBot) -> HandlerResult {
    bot.send_message(msg.chat.id, info.test).await?;
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn mode(bot: Bot, msg: Message) -> HandlerResult {
    // todo. 현재 모드 정보 조회
    let result :&str = "";
    bot.send_message(msg.chat.id, format!("현재 모드 : {}", result)).await?;
    Ok(())
}

async fn switch(bot: Bot, msg: Message) -> HandlerResult {
    // todo. DB에서 현재 모드 변경
    let result :&str = "";
    bot.send_message(msg.chat.id, format!("모드 변경 성공. 현재 모드 : {}", result)).await?;
    Ok(())
}

async fn update(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    // todo
    // 1. 크롤링
    // 2. DB에서 완료 덱 가져와
    // 3. 미완료된 덱 중에서 일반덱 / 특수덱으로 분류 해
    // 두 개의 iter()로 생성

    bot.send_message(msg.chat.id, "다음 일반 덱")
       .reply_markup(InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback("일반 덱 1", "일반 덱 1")]]))
       .await?;
    bot.send_message(msg.chat.id, "잔여 특수 덱")
       .reply_markup(InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback("특수 덱 1", "특수 덱 1")],
        [InlineKeyboardButton::callback("특수 덱 2", "특수 덱 2")],]))
       .await?;
    dialogue.update(State::Challenge).await?;
    Ok(())
}

async fn reset(bot: Bot, msg: Message) -> HandlerResult {
    // todo. DB에 저장된 이력 모두 삭제
    let mode :&str = "";
    bot.send_message(msg.chat.id, format!("모드 {}에 대한 이력 삭제 완료", mode)).await?;
    Ok(())
}

// memo. iter-map 안에서는 비동기를 날리지 못 함
async fn done(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    // todo. DB에 저장된 이력 모두 조회
    let mode :Vec<String> = vec![];
    // 버튼 보내기
    bot.send_message(msg.chat.id, "완료 내역")
       .reply_markup(InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback("SAMPLE 완료 1", "완료 1")],
            [InlineKeyboardButton::callback("SAMPLE 완료 2", "완료 2")]]))
       .await?;
    dialogue.update(State::Rollback).await?;
    Ok(())
}

async fn fix(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "css path 수정 완료").await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "잘못된 커맨드").await?;
    Ok(())
}

async fn success(bot: Bot, 
                dialogue: MyDialogue,
                q: CallbackQuery, ) -> HandlerResult 
{
    if let Some(deck) = &q.data {
        bot.send_message( dialogue.chat_id(), format!("{} 완료!", deck)).await?;
        dialogue.exit().await?;
    }
    Ok(())
}

async fn rollback(bot: Bot, 
                dialogue: MyDialogue,
                q: CallbackQuery, ) -> HandlerResult 
{
    if let Some(deck) = &q.data {
        bot.send_message(dialogue.chat_id(), format!("{} 롤백", deck)).await?;
        dialogue.exit().await?;
    }
    Ok(())
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
        run(TOKEN).await;
    }
}

