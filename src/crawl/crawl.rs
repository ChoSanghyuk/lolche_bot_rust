use reqwest::blocking::get;
use scraper::{Html, Selector};

pub struct Crawler {
    main_url: String,
	pbe_url:  String,
	css_path: String,
}

impl Crawler {
    pub fn new() -> Self {
        Self {
            main_url: String::from(""),
            pbe_url : String::from(""),
            css_path : update_css_path("", "").unwrap(), // TODO. URL들 다 config로 빼고 주입 받기
        }
    }

}

fn update_css_path(url: &str, target: &str) -> Option<String> {
	let path = css_path(url, target).unwrap();
	let result = crawl(url, &path).unwrap();
   
   if result.len() == 0 {
		return None
   }
   
   return Some(path)
}


fn css_path(url: &str, target:&str) -> Result<String, String> {
    Ok(String::from("Sample"))
}

fn crawl(url: &str, path: &str) -> Result<Vec<String>, String> {

    // Fetch the URL content
    let response = get(url)?.text()?;

    // Parse the HTML document
    let document = Html::parse_document(&response);

    // Create a selector for the CSS path
    let selector = Selector::parse(css_selector)?;

    // Find and iterate over matching elements
    for element in document.select(&selector) {
        // Extract and print the element's inner text
        let text = element.text().collect::<Vec<_>>().join(" ");
        println!("{}", text);
    }

    Ok(vec![String::from("Result")])
}


#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test(){
		let crawler = Crawler::new();
		print!("{}", crawler.css_path)
	}
}