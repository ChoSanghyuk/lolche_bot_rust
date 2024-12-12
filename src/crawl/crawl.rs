use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::{error::Error, fmt::format};
use regex::Regex;

use super::error::CrawlError;

struct CssPathFinder {
    tag: Regex,
    class : Regex,
    id: Regex,
}
pub struct Crawler {
    main_url: &'static str,
	pbe_url:  &'static str,
	css_path: String,
    path_finder: CssPathFinder,
}

impl Crawler {
    pub fn new() -> Self {
        let crawler = Self { // TODO. URL들 다 config로 빼고 주입 받기
            main_url: "https://lolchess.gg/meta",
            pbe_url : "https://lolchess.gg/meta?pbe=true",
            css_path : String::from("#content-container > section > div.css-s9pipd.e2kj5ne0 > div > div > div > div.css-5x9ld.emls75t2 > div.css-35tzvc.emls75t4 > div"), 
            path_finder : CssPathFinder { 
                    tag: Regex::new(r"^[^<]*<([^>]+)>.*$").unwrap(), 
                    class: Regex::new(r#"id="([^"]+)""#).unwrap(), 
                    id: Regex::new(r#"class="([^"]+)""#).unwrap(),
                }
        };
        crawler
    }

    pub fn update_css_path(&self, url: &str, target: &str) -> Result<String, CrawlError> {
        let path = self.path_finder.css_path(url, target).unwrap();
        
        if crawl(url, &path).is_err() {
            Err(format!("잘못된 css path 결과"))?
        }
       
       return Ok(path)
    }

    pub fn main_dec(&self) -> Result<Vec<String>, CrawlError> {
        crawl(self.main_url, &self.css_path)
    }

    pub fn pbe_dec(&self) -> Result<Vec<String>, CrawlError> {
        crawl(self.pbe_url, &self.css_path)
    }
}

impl CssPathFinder {

    fn css_path(&self, url: &str, target:&str) -> Result<String, String> {

        // Parse the HTML content
        let document = document(url).unwrap(); // todo
        // Define a basic selector that selects all elements
        let selector = Selector::parse("div:not(:has(*))").unwrap();
    
        let mut path = String::new();
        // Traverse all elements to find one that contains the target string
        for element in document.select(&selector) {
            let element_text = element.text().collect::<Vec<_>>().join(" ");
    
            if element_text.contains(target) {
                let tag = format!("{:?}", element.value());
                path = self.css_format_converter(&tag).unwrap();
                
                for ancestor in element.ancestors() {
                    let ancestor_tag = format!("{:?}", ancestor.value());    
                    if let Some(css_format) = self.css_format_converter(&ancestor_tag) {
                        path = format!("{} > {}", css_format, path)
                    }
                } 
            }
        }
        Ok(path)
    }

    fn css_format_converter(&self, input:&str) -> Option<String> {

        let row = self.tag.captures(input)
                                        .map(|cap| cap[1].to_string())?;
    
        let tag= row.split(" ").collect::<Vec<_>>()[0];
        let class_selector = self.class
                                                    .captures(&row)
                                                    .map(|cap| cap[1].split_whitespace().collect::<Vec<_>>().join(".") );
                                                
        let id_selector = self.id
                                            .captures(&row)
                                            .map(|cap| cap[1].to_string());
    
        
        Some(format!("{}{}{}", 
                    tag, 
                    id_selector.map(|id| format!("#{}",id)).unwrap_or_default(),
                    class_selector.map(|class| format!(".{}", class)).unwrap_or_default())
            )
    }
}



fn document(url: &str) -> Result<Html, CrawlError> {
    // Fetch the URL content
    let client = reqwest::blocking::Client::new();
    let response = client.get(url)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .map_err(|e| format!("Fail to get url. {:?}", e))?
        .text()
        .map_err(|e| format!("Fail to get response text. {:?}", e))?;


    // Return parsed HTML document
    Ok(Html::parse_document(&response))
}

fn crawl(url: &str, path: &str) -> Result<Vec<String>, CrawlError> {

    // Parse the HTML document
    let document = document(url)?;

    // Create a selector for the CSS path
    let selector = Selector::parse(path)
                            .map_err(|e| format!("Fail to parse {:?}", e))?;

    let mut result: Vec<String> = Vec::new();
    // Find and iterate over matching elements
    for element in document.select(&selector) {
        // Extract and print the element's inner text
        result.push(element.text().collect::<Vec<_>>().join(" "));
    }

    if result.len() == 0 {
        return Err(format!("조회 결과 없음 오류"))? 
   }

    Ok(result)
}


#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test(){
		let crawler = Crawler::new();
		print!("{}", crawler.css_path)
	}

    #[test]
    fn crawl_test(){
        let url = "https://lolchess.gg/meta";
        // let path = "div#content-container > section.css-1v8my8o.esg9lhj0 > div.css-s9pipd.e2kj5ne0 > div > div.css-1iudmso.emls75t0 > div.css-1r1x0j5.emls75t1 > div.css-5x9ld.emls75t2 > div.css-35tzvc.emls75t4 > div" ;
        let path = "html.b-dakgg > body > div#__next > div.theme-dark.css-q3savf.e19bnpjr0 > div.css-1x48m3k.eetc6ox0 > div.content > div.css-vwmdp.e18pwoek0 > div.main-contents > div#content-container.css-nys28y.e18pwoek4 > section.css-1v8my8o.esg9lhj0 > div.css-s9pipd.e2kj5ne0 > div > div.css-1iudmso.emls75t0 > div.css-1r1x0j5.emls75t1 > div.css-5x9ld.emls75t2 > div.css-35tzvc.emls75t4 > div";
        let result = crawl(url, path);
        match result {
            Ok(_) => print!("{:?}", result),
            Err(_) => assert!(false),
        }
        // is_err를 사용해서 테스트 하면 간결. assert!(crawl(url, path).is_err());
    }

    #[test]
    fn get_test() {
        let url = "https://lolchess.gg/meta"; // Replace with your URL

        let client = reqwest::blocking::Client::new();
        let res = client.get(url)
            .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .send().unwrap()
            .text().unwrap();
        
        print!("{}", res)
    }

    #[test]
    fn crawler_test() {
        let crawler = Crawler::new();
        match crawler.main_dec() {
            Ok(result) => {
                print!("{:?}", result)
            }
            Err(e) => {
                eprint!("{}", e)
            }
        }
    }
    /*
    div#content-container > section.css-1v8my8o.esg9lhj0 > div.css-s9pipd.e2kj5ne0 > div > div.css-1iudmso.emls75t0 > div.css-1r1x0j5.emls75t1 > div.css-5x9ld.emls75t2 > div.css-35tzvc.emls75t4 > div
     */
    #[test]
    fn find_selector_test(){
        let crawler = Crawler::new();
        
        if let Ok(result) = crawler.path_finder.css_path(&crawler.main_url, "초반 빌드업 요약") {
            print!("{}", result)
        } else {
            assert!(false)
        }
        
    }
}