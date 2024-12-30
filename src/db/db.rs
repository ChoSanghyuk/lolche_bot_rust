use mysql::*;
use mysql::prelude::*;
use crate::bot::traits::Mode;

#[derive(Clone)]
pub struct Storage {
    pool: Pool
}

impl Storage {

    pub fn new(url:&str) -> Self {
        let pool = Pool::new(url).unwrap();
        Self { pool: pool }
    }
    
    fn create(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        let mut conn = self.pool.get_conn()?;
        conn.query_drop(r"
            CREATE TABLE IF NOT EXISTS mains (
            id 	INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(30) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT NOW()
        )")?;

        conn.query_drop(r"
            CREATE TABLE IF NOT EXISTS pbes (
            id 	INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(30) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT NOW()
        )")?;

        conn.query_drop(r"
            CREATE TABLE IF NOT EXISTS modes (
            id 	INT PRIMARY KEY,
            is_main BOOL
        )")?;

        Ok(())
    }

    pub fn record_done(&self, input:&str, mode: &Mode) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match mode {
            Mode::main => self.insert_main(input),
            Mode::pbe => self.insert_pbe(input)
        }
    }
    fn insert_main(&self, input:&str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            INSERT INTO mains (name) 
            VALUES (:dec_name)",
             (input, ))?; // memo. 파라미터가 하나일 때에는 (파라미터1, )으로 사용 
        Ok(())
    }

    fn insert_pbe(&self, input:&str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            INSERT INTO pbes (name) 
            VALUES (:dec_name)",
             (input, ))?; // memo. 파라미터가 하나일 때에는 (파라미터1, )으로 사용 
        Ok(())
    }
    
    pub fn upsert_mode(&self, mode: &Mode) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        let is_main :bool ;

        match mode {
            Mode::main => is_main = true,
            Mode::pbe => is_main = false,
        }

        conn.exec_drop(r"
            INSERT INTO modes (id, is_main) 
            VALUES (:id, :is_main) 
            ON DUPLICATE KEY UPDATE
            is_main = :is_main",
            (1, is_main, is_main) // memo. only supports positional placeholders
        )?;
        Ok(())
    }

    pub fn delete_all(&self, mode:&Mode) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        match *mode {
            Mode::main => self.delete_main(),
            Mode::pbe => self.delete_pbe(),
        }
    }

    fn delete_main(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            DELETE FROM mains
            WHERE 1=1",
            () // memo. only supports positional placeholders
        )?;
        Ok(())
    }

    fn delete_pbe(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            DELETE FROM pbes
            WHERE 1=1",
            () // memo. only supports positional placeholders
        )?;
        Ok(())
    }

    pub fn delete_record(&self, mode:&Mode, target:&str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match *mode {
            Mode::main => self.delete_main_record(target),
            Mode::pbe => self.delete_pbe_record(target),
        }
    }

    fn delete_main_record(&self, target:&str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            DELETE FROM mains
            WHERE 1=1
            AND name = :name",
            (target,) // memo. only supports positional placeholders
        )?;
        Ok(())
    }

    fn delete_pbe_record(&self, target:&str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            DELETE FROM pbes
            WHERE 1=1
            AND name = :name",
            (target,) // memo. only supports positional placeholders
        )?;
        Ok(())
    }

    pub fn retrieve_done(&self, mode: &Mode) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        match *mode {
            Mode::main => self.select_main(),
            Mode::pbe => self.select_pbe(),
        }
    }

    fn select_main(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        let result: Vec<String> = conn.exec(r"
            SELECT name
            FROM mains
            WHERE 1=1",
           ()
        )?;
        Ok(result)
    }

    fn select_pbe(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get_conn()?;
        let result: Vec<String> = conn.exec(r"
            SELECT name
            FROM pbes
            WHERE 1=1",
           ()
        )?;
        Ok(result)
    }

    pub fn select_mode(&self) -> Result<Mode,  Box<dyn std::error::Error + Send + Sync>> {

        let mut conn = self.pool.get_conn()?;
        let result: Option<bool> = conn.exec_first(r"
            SELECT is_main
            FROM modes
            WHERE id=1",
           ()
        )?;

        match result {
            Some(is_main) => {
                if is_main {
                    Ok(Mode::main)
                } else {
                    Ok(Mode::pbe)   
                }
            }
            None => Ok(Mode::main)
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;

    const url: &'static str = "mysql://root:root@127.0.0.1:3306/lolche";

	#[test]
	fn create_test(){
		let stg = Storage::new(url);
		match stg.create() {
            Ok(_) => print!("Success"),
            Err(e) => {
                eprint!("{}", e);
                assert!(false);
            }
        }
	}

    #[test]
    fn insert_test(){
        let stg = Storage::new(url);   
        match stg.insert_main("[상징] 6자동기계 코그모 리롤덱") {
            Ok(_) => print!("Success"),
            Err(e) => {
                eprint!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn upsert_mode_test(){
        let stg = Storage::new(url);   
        match stg.upsert_mode(&Mode::main) {
            Ok(_) => print!("Success"),
            Err(e) => {
                eprint!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn select_main_test(){
        let stg = Storage::new(url);   
        match stg.select_main() {
            Ok(result) => print!("{:?}", result),
            Err(e) => {
                eprint!("{}", e);
                assert!(false);
            }
         }
    }

    #[test]
    fn delete_main_test(){
        let stg = Storage::new(url);   
        match stg.delete_main() {
            Ok(result) => print!("{:?}", result),
            Err(e) => {
                eprint!("{}", e);
                assert!(false);
            }
         }
    }

    #[test]
    fn test_pool_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Pool>();
    }
}