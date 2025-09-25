mod binance;
mod olhc_forecast;
mod telegram;

use std::thread;
use std::time::Duration;

use dotenvy::dotenv;

use polars::prelude::*;
use std::env;

use olhc_forecast::OhlcForecast;

fn main() -> PolarsResult<()> {
    dotenv().ok(); // Load environment variables from .env

    let sleep: u64 = env::var("SLEEP_SECONDS")
        .unwrap_or("1800".into())
        .parse()
        .unwrap();

    let forecast = OhlcForecast::new();
    loop {
        let _ = forecast.run()?;
        println!("Sleeping for {} seconds...", sleep);
        thread::sleep(Duration::from_secs(sleep));
    }
}
