use mysql::*;
use mysql::prelude::*;
use crate::mode;

pub struct Storage {
    pool: Pool
}

impl Storage {

    fn new() -> Self {
        let url = "mysql://root:root@127.0.0.1:3306/lolche";
        let pool = Pool::new(url).unwrap();
        Self { pool: pool }

    }
    
    fn create(&self) -> Result<(), Box<dyn std::error::Error>> {

        let mut conn = self.pool.get_conn()?;
        conn.query_drop(r"
            CREATE TABLE IF NOT EXISTS main (
            id 	INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(30) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT NOW()
        )")?;

        conn.query_drop(r"
            CREATE TABLE IF NOT EXISTS pbe (
            id 	INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(30) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT NOW()
        )")?;

        conn.query_drop(r"
            CREATE TABLE IF NOT EXISTS mode (
            id 	INT PRIMARY KEY,
            is_main BOOL
        )")?;

        Ok(())
    }

    fn insert_main(&self, input:&str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            INSERT INTO main (name) 
            VALUES (:dec_name)",
             (input, ))?; // memo. 파라미터가 하나일 때에는 (파라미터1, )으로 사용 
        Ok(())
    }

    fn insert_pbe(&self, input:&str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            INSERT INTO pbe (name) 
            VALUES (:dec_name)",
             (input, ))?; // memo. 파라미터가 하나일 때에는 (파라미터1, )으로 사용 
        Ok(())
    }
    
    fn upsert_mode(&self, mode: mode::Lolche) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let mut is_main :bool ;

        match mode {
            mode::Lolche::main => is_main = true,
            mode::Lolche::pbe => is_main = false,
        }

        conn.exec_drop(r"
            INSERT INTO mode (id, is_main) 
            VALUES (:id, :is_main) 
            ON DUPLICATE KEY UPDATE
            is_main = :is_main",
            (1, is_main, is_main) // memo. only supports positional placeholders
        )?;
        Ok(())
    }

    fn delete_main(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            DELETE FROM main
            WHERE 1=1",
            () // memo. only supports positional placeholders
        )?;
        Ok(())
    }

    fn delete_pbe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(r"
            DELETE FROM pbe
            WHERE 1=1",
            () // memo. only supports positional placeholders
        )?;
        Ok(())
    }

    fn select_main(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let result: Vec<String> = conn.exec(r"
            SELECT name
            FROM main
            WHERE 1=1",
           ()
        )?;
        Ok(result)
    }

    fn select_pbe(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let result: Vec<String> = conn.exec(r"
            SELECT name
            FROM pbe
            WHERE 1=1",
           ()
        )?;
        Ok(result)
    }

    fn select_mode(&self) -> Result<mode::Lolche,  Box<dyn std::error::Error>> {

        let mut conn = self.pool.get_conn()?;
        let result: Option<bool> = conn.exec_first(r"
            SELECT is_main
            FROM mode
            WHERE id=1",
           ()
        )?;

        match result {
            Some(is_main) => {
                if is_main {
                    Ok(mode::Lolche::main)
                } else {
                    Ok(mode::Lolche::pbe)   
                }
            }
            None => Ok(mode::Lolche::main)
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;

	#[test]
	fn create_test(){
		let stg = Storage::new();
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
        let stg = Storage::new();   
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
        let stg = Storage::new();   
        match stg.upsert_mode(mode::Lolche::main) {
            Ok(_) => print!("Success"),
            Err(e) => {
                eprint!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn select_main_test(){
        let stg = Storage::new();   
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
        let stg = Storage::new();   
        match stg.delete_main() {
            Ok(result) => print!("{:?}", result),
            Err(e) => {
                eprint!("{}", e);
                assert!(false);
            }
         }
    }
}