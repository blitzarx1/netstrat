use std::cmp::Ordering;

use serde::Deserialize;
use serde_json;

use crate::network::rest::Rest;
use crate::sources::binance::interval::Interval;

#[derive(Clone, Debug)]
pub struct Client {}

const BASE_URL: &str = "https://api.binance.com";
const PATH_KLINE: &str = "/api/v3/klines";
const PATH_INFO: &str = "/api/v3/exchangeInfo";

#[derive(Debug, Deserialize, Default)]
pub struct Info {
    pub symbols: Vec<Symbol>,
}

#[derive(Deserialize)]
struct KlineData(
    i64, // Open time
    f32, // Open
    f32, // High
    f32, // Low
    f32, // Close
    f32, // Volume
    i64, // Close time
    f32, // Quote asset volume
    i64, // Number of trades
    f32, // Taker buy base asset volume
    f32, // Taker buy quote asset volume
    f32, // Ignore
);

#[derive(Debug, Deserialize, Default)]
pub struct Symbol {
    pub symbol: String,
    pub status: String,

    #[serde(rename = "baseAsset")]
    base_asset: String,

    #[serde(rename = "baseAssetPrecision")]
    base_asset_precision: usize,

    #[serde(rename = "quoteAsset")]
    quote_asset: String,

    #[serde(rename = "quotePrecision")]
    quote_precision: usize,

    #[serde(rename = "quoteAssetPrecision")]
    quote_asset_precision: usize,

    #[serde(rename = "baseCommissionPrecision")]
    base_commission_precision: usize,

    #[serde(rename = "quoteCommissionPrecision")]
    quote_commission_precision: usize,

    #[serde(rename = "icebergAllowed")]
    iceberg_allowed: bool,

    #[serde(rename = "ocoAllowed")]
    oco_allowed: bool,

    #[serde(rename = "quoteOrderQtyMarketAllowed")]
    quote_order_qty_market_allowed: bool,

    #[serde(rename = "allowTrailingStop")]
    allow_trailing_stop: bool,

    #[serde(rename = "isSpotTradingAllowed")]
    is_spot_trading_allowed: bool,

    #[serde(rename = "isMarginTradingAllowed")]
    is_margin_trading_allowed: bool,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Kline {
    pub t_open: i64,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
    pub t_close: i64,
    pub quote_asset_volume: f32,
    pub number_of_trades: i64,
    pub taker_buy_base_asset_volume: f32,
    pub taker_buy_quote_asset_volume: f32,
}

impl Kline {
    fn from_kline_data(data: KlineData) -> Self {
        Kline {
            t_open: data.0,
            open: data.1,
            high: data.2,
            low: data.3,
            close: data.4,
            volume: data.5,
            t_close: data.6,
            quote_asset_volume: data.7,
            number_of_trades: data.8,
            taker_buy_base_asset_volume: data.9,
            taker_buy_quote_asset_volume: data.10,
        }
    }
}

impl Ord for Kline {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.close < other.close {
            return Ordering::Less;
        }

        if self.close > other.close {
            return Ordering::Greater;
        }

        Ordering::Equal
    }
}

impl Eq for Kline {}

impl PartialOrd for Kline {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.close.partial_cmp(&other.close)
    }
}

impl Client {
    pub async fn kline(
        symbol: String,
        interval: Interval,
        start_time: i64,
        limit: usize,
    ) -> Result<Vec<Kline>, Box<dyn std::error::Error>> {
        let url = format!("{}{}", BASE_URL, PATH_KLINE);
        let params = &[
            ("symbol", symbol.as_str()),
            ("interval", interval.as_str()),
            ("startTime", &start_time.to_string()),
            ("limit", &limit.to_string()),
        ];
        let resp = Rest::new().get_with_params_blocking(&url, params).await?;
        let json_str = &resp.text().await?;
        let res: Vec<KlineData> = serde_json::from_str(json_str)?;

        Ok(res
            .into_iter()
            .map(|data| Kline::from_kline_data(data))
            .collect())
    }

    pub async fn info() -> Result<Info, Box<dyn std::error::Error>> {
        let url = format!("{}{}", BASE_URL, PATH_INFO);
        let resp = Rest::new().get(&url).await?;
        let json_str = &resp.text().await?;
        let res: Info = serde_json::from_str(json_str)?;
        Ok(res)
    }

    pub fn info_blocking() -> Result<Info, Box<dyn std::error::Error>> {
        let url = format!("{}{}", BASE_URL, PATH_INFO);
        let resp = Rest::new().get_blocking(&url);
        let res: Info = serde_json::from_str(resp.unwrap().text().unwrap().as_str())?;
        Ok(res)
    }
}
