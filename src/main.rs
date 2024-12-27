mod crawl; 
mod db;
mod bot;
mod config;

use crawl::crawl::LolcheggCrawler;
use db::db::Storage;
use bot::bot::LolcheBot;
use config::conf::Config;

#[tokio::main]
async fn main() {
    
    let config = Config::new();

    std::env::set_var("RUST_LOG", config.log_level());
    pretty_env_logger::init();


    let lolchegg_crawler = LolcheggCrawler::new();
    let stg = Storage::new(&config.db_url()); // memo. config.db_url()의 결과값이 String을 소유하고 있으며, main 블록이 끝나면 소멸됨

    
    let my_bot = LolcheBot::new(config.token(), lolchegg_crawler,stg);

    log::info!("Lolche Bot Started!");

    my_bot.run().await;
}
