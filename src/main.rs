mod crawl; 
mod db;
mod bot;

use crawl::crawl::LolcheggCrawler;
use db::db::Storage;
use bot::bot::LolcheBot;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    const TOKEN: &str = "";
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    log::info!("Test Start");

    let my_bot = LolcheBot{
        loader : LolcheggCrawler::new(),
        stg: Storage::new(),
    };

    my_bot.run(TOKEN).await;
}

/*
todo
config 파일에서 가져오기
에러 메시지로 보내기
*/