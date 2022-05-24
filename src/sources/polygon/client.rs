use crate::network::rest::Rest;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default, Serialize, Debug)]
#[serde(default)]
pub struct Info {
    results: Vec<InfoResult>,
    status: String,
    request_id: String,
    count: u32,
    error: String,
}

#[derive(Deserialize, Default, Serialize, Debug)]
#[serde(default)]
struct InfoResult {
    id: u32,
    #[serde(rename = "type")]
    item_type: String,
    asset_class: String,
    locale: String,
    url: String,
}

#[derive(Deserialize, Default, Serialize, Debug)]
#[serde(default)]
pub struct ReferenceTickers {
    results: Vec<ReferenceTickersResult>,
    status: String,
    request_id: String,
    count: u32,
    next_url: String,
    error: String,
}

#[derive(Deserialize, Default, Serialize, Debug)]
#[serde(default)]
struct ReferenceTickersResult {
    ticker: String,
    name: String,
    market: String,
    locale: String,
    primary_exchange: String,
    #[serde(rename = "type")]
    ticker_type: String,
    active: bool,
    currency_name: String,
    cik: String,
    last_updated_utc: String,
}

pub struct Client {
    c: Rest,
    api_key: String,
}

impl Client {
    pub fn new(api_key: String) -> Client {
        Client {
            c: Rest::new(),
            api_key,
        }
    }

    async fn get_blocking(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let params = &[("apiKey", <&str>::from(&self.api_key[..]))];
        self.c.get_with_params(url, params).await
    }

    async fn get_with_params(
        &self,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response, reqwest::Error> {
        let params = &[&[("apiKey", <&str>::from(&self.api_key[..]))], params].concat();
        self.c.get_with_params(url, params).await
    }

    pub async fn info(&self) -> Result<Info, Box<dyn std::error::Error>> {
        let resp = self
            .get_with_params(
                "https://api.polygon.io/v3/reference/exchanges",
                &[("asset_class", "crypto")],
            )
            .await?;

        let json_str = &resp.text().await?;
        let res: Info = serde_json::from_str(json_str)?;
        Ok(res)
    }

    pub async fn reference_tickers(&self) -> Result<ReferenceTickers, Box<dyn std::error::Error>> {
        let resp = self
            .get_blocking("https://api.polygon.io/v3/reference/tickers")
            .await?;

        let json_str = &resp.text().await?;
        let res: ReferenceTickers = serde_json::from_str(json_str)?;
        Ok(res)
    }
}
