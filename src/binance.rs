use polars::prelude::*;
use reqwest::blocking::Client;
use serde_json::Value;

pub struct Binance {
    client: Client,
    interval: String,
    limit: i32,
}

impl Binance {
    pub fn new() -> Self {
        let interval = std::env::var("INTERVAL").unwrap_or("1h".into());
        let limit: i32 = std::env::var("LIMIT")
            .unwrap_or("1000".into())
            .parse()
            .unwrap();

        Binance {
            client: Client::new(),
            interval,
            limit,
        }
    }

    pub fn fetch(&self, symbol: &str) -> PolarsResult<DataFrame> {
        let url = "https://api.binance.com/api/v3/klines";
        let data: Vec<Vec<Value>> = self
            .client
            .get(url)
            .query(&[
                ("symbol", symbol),
                ("interval", &self.interval),
                ("limit", &self.limit.to_string()),
            ])
            .send()
            .unwrap()
            .json()
            .unwrap();

        let mut opens = Vec::new();
        let mut highs = Vec::new();
        let mut lows = Vec::new();
        let mut closes = Vec::new();
        let mut volumes = Vec::new();

        for row in data {
            opens.push(row[1].as_str().unwrap().parse::<f64>().unwrap());
            highs.push(row[2].as_str().unwrap().parse::<f64>().unwrap());
            lows.push(row[3].as_str().unwrap().parse::<f64>().unwrap());
            closes.push(row[4].as_str().unwrap().parse::<f64>().unwrap());
            volumes.push(row[4].as_str().unwrap().parse::<f64>().unwrap());
        }

        let df = df![
            "open" => opens,
            "high" => highs,
            "low" => lows,
            "close" => closes,
            "volume" => volumes
        ]?;
        Ok(df)
    }
}
