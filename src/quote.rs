use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Error};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Quote {
    pub quote: String,
    pub author: String
}

pub async fn get_quote(api: &str) -> Result<Quote, Error> {
    let client = Client::new();
    let url = "https://api.dailyquotes.dev/api/quotes/motivational";
    let res = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", api))
        .header(CONTENT_TYPE, "application/json")
        .send().await;
    let quote = res?.json::<Quote>().await;
    return quote;
}
