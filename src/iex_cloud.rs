use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use moka::future::Cache;
use std::time::Duration;

const IEX_URL: &str = "https://cloud.iexapis.com/v1";

// TODO which fields should we be using?

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct QuoteResponse {
    pub symbol: String,
    pub change: f32,
    pub change_percent: f32,
    pub delayed_price: f32,
    pub close: f32,
}

pub struct Client {
    client: ReqwestClient,
    token: String,
    cache: Cache<String, QuoteResponse>,
}

// TODO error handling all over the place

impl Client {
    pub fn new(token: String) -> Self {
        Client {
            client: ReqwestClient::new(),
            token,
            cache: moka::future::CacheBuilder::new(10_000)
                .time_to_live(Duration::from_secs(15 * 60))
                .time_to_idle(Duration::from_secs(1 * 60))
                .build(),
        }
    }

    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client.get(path).query(&[("token", &self.token)]).send().await.unwrap()
    }

    pub async fn get_quote(&self, symbol: String) -> QuoteResponse {
        let url = format!(
            "{base}/stock/{symbol}/quote",
            base = IEX_URL,
            symbol = symbol,
        );
        match self.cache.get(&symbol) {
            Some(resp) => resp,
            None => {
                let resp = self.get(&url).await.json::<QuoteResponse>().await.unwrap();
                self.cache.insert(symbol, resp.clone()).await;
                resp
            }
        }
    }
}

// TODO: tests
