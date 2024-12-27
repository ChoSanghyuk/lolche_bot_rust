use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    database : Database,
    bot : Bot,
    app : App,
}

#[derive(Debug, Deserialize)]
struct Database {
    db: String,
    ip: String, 
    port: u32,
    user: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Bot {
    token: String
}

#[derive(Debug, Deserialize)]
struct App {
    log : String
}


impl Config {
    pub fn new() -> Self {
        let config_content = fs::read_to_string("./src/config/config.yaml").unwrap();
        let config: Config = serde_yaml::from_str(&config_content).unwrap();
        config
    }

    pub fn token(&self) -> String {
        self.bot.token.clone()
    }

    pub fn db_url(&self) -> String {
        format!("mysql://{}:{}@{}:{}/{}",
            self.database.user,
            self.database.password,
            self.database.ip,
            self.database.port,
            self.database.db,
        )
    }

    pub fn log_level(&self) -> &str {
        &self.app.log
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    #[test]
    fn check_current_path() {
        match env::current_dir() {
            Ok(path) => println!("Current working directory: {}", path.display()),
            Err(e) => eprintln!("Error getting current directory: {}", e),
        }
    }
    #[test]
    fn config_print() {

        let config_content = fs::read_to_string("./src/config/config.yaml").unwrap();
    
        let config: Config = serde_yaml::from_str(&config_content).unwrap();
        
        println!("{:#?}", config);
    }
}