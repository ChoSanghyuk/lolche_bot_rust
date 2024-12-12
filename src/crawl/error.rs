#[derive(Debug)]
pub struct CrawlError(String);

impl std::fmt::Display for CrawlError {
    // '_는 익명 수명을 나타내며, 컴파일러가 자동으로 추론하는 수명을 명시적으로 나타냄
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CrawlError{}

impl From<String> for CrawlError {
    fn from(msg: String) -> Self{
        Self(msg)
    }
}
