use chrono::Utc;

use binance::Binance;
use ndarray::Array2;
use polars::prelude::IndexOrder;
use polars::prelude::*;
use smartcore::ensemble::random_forest_regressor::{
    RandomForestRegressor, RandomForestRegressorParameters,
};
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::metrics::r2;
use smartcore::model_selection::train_test_split;
use std::env;
use telegram::Telegram;

use crate::{binance, telegram};

pub struct OhlcForecast {
    symbols: Vec<String>,
    threshold: f64,
}

impl OhlcForecast {
    pub fn new() -> Self {
        let symbols_str = env::var("SYMBOLS").unwrap_or_else(|_| "DOTUSDT".to_string());
        let symbols: Vec<String> = symbols_str
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .collect();
        let threshold: f64 = env::var("PRICE_CHANGE_THRESHOLD")
            .unwrap_or_else(|_| "2".to_string())
            .parse()
            .unwrap_or(2.0);

        if symbols.is_empty() {
            println!("[WARN] The symbols is missing in .env");
        }

        Self { symbols, threshold }
    }

    fn build_features(&self, mut df: DataFrame) -> PolarsResult<DataFrame> {
        df = df
            .lazy()
            .with_column((col("open") - col("close")).alias("oc_diff"))
            .with_column((col("high") - col("low")).alias("hl_diff"))
            .with_column(
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 5,
                        min_periods: 1,
                        weights: None,
                        center: false,
                        fn_params: None,
                    })
                    .alias("ma_5"),
            )
            .with_column(
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 10,
                        min_periods: 1,
                        weights: None,
                        center: false,
                        fn_params: None,
                    })
                    .alias("ma_10"),
            )
            .with_column(col("volume").pct_change(lit(1)).alias("volume_change"))
            .collect()?;

        Ok(df)
    }

    fn prepare_training_data(
        &self,
        mut df: DataFrame,
    ) -> PolarsResult<(DenseMatrix<f64>, Vec<f64>, Vec<String>)> {
        // Create target column
        df = df
            .lazy()
            .with_column(col("close").shift(lit(-1)).alias("target"))
            .collect()?;

        // Drop rows with nulls
        df = df.drop_nulls::<String>(None)?;

        // Select features
        let feature_cols: Vec<String> = df
            .get_column_names()
            .iter()
            .filter(|&name| AsRef::<str>::as_ref(name) != "target")
            .map(|name| name.to_string())
            .collect();

        // Convert features to ndarray
        let x_nd: Array2<f64> = df
            .select(&feature_cols)?
            .to_ndarray::<Float64Type>(IndexOrder::C)?;

        // Target column
        let y: Vec<f64> = df
            .select(["target"])?
            .column("target")?
            .f64()?
            .into_no_null_iter()
            .collect();

        // Flatten ndarray into Vec<f64>
        let rows = x_nd.nrows();
        let cols = x_nd.ncols();
        let (flat, _offset) = x_nd.into_raw_vec_and_offset();

        // ✅ smartcore 0.4.2 uses from_array (takes &[f64])
        let x_matrix = DenseMatrix::new(rows, cols, flat, false)
            .map_err(|e| PolarsError::ComputeError(format!("DenseMatrix error: {:?}", e).into()))?;

        Ok((x_matrix, y, feature_cols))
    }

    // Helper function to prepare the last row of features for prediction.
    fn prepare_last_features(
        &self,
        df: DataFrame,
        feature_cols: &[String],
    ) -> PolarsResult<DenseMatrix<f64>> {
        // Take the last row
        let df_last = df.tail(Some(1));

        // Convert to ndarray (row-major order)
        let x_last = df_last
            .select(feature_cols)?
            .to_ndarray::<Float64Type>(IndexOrder::C)?;

        let (rows, cols) = x_last.dim();
        let flat: Vec<f64> = x_last.iter().copied().collect();

        // Create DenseMatrix (SmartCore expects row-major = false means column-major? we’ll pass false here)
        let x_matrix = DenseMatrix::new(rows, cols, flat, false)
            .map_err(|e| PolarsError::ComputeError(format!("{:?}", e).into()))?;

        Ok(x_matrix)
    }

    pub fn run(&self) -> PolarsResult<()> {
        let tg = Telegram::new();
        let binance = Binance::new();

        for symbol in &self.symbols {
            match binance.fetch(&symbol) {
                Ok(df) => {
                    // println!("{:?}", df);
                    let df_feat = self.build_features(df)?;
                    if df_feat.height() < 50 {
                        println!("[WARN] Not enough data for {}", symbol);
                        continue;
                    }
                    let (x_matrix, y, feature_cols) =
                        self.prepare_training_data(df_feat.clone())?;

                    // ✅ SmartCore built-in split (80% train, 20% test)
                    let (x_train, x_test, y_train, y_test) =
                        train_test_split(&x_matrix, &y, 0.2, true, None);
                    let model = RandomForestRegressor::fit(
                        &x_train,
                        &y_train,
                        RandomForestRegressorParameters::default(),
                    )
                    .unwrap();

                    let y_pred = model.predict(&x_test).unwrap();
                    let r2 = r2(&y_test, &y_pred);
                    println!("[INFO] Validation R² score for {}: {:.4}", symbol, r2);

                    // Forecast next close
                    let last_features =
                        self.prepare_last_features(df_feat.clone(), &feature_cols)?;
                    let predicted_price = model.predict(&last_features).unwrap()[0];

                    let current_price = df_feat.column("close")?.f64()?.last().unwrap_or(0.0);

                    let change_pct = (predicted_price - current_price) / current_price * 100.0;

                    let mut msg = format!(
                        "📢 📢 📢 *ML Forecast for #{symbol}*\n\
                        Time: {}\n\
                        Current Price: {:.4}\n\
                        Next Price: {:.4}\n\
                        Change: {:.2}%\n",
                        Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                        current_price,
                        predicted_price,
                        change_pct
                    );

                    if change_pct > self.threshold {
                        msg += "Signal: *Buy ✅*";
                    } else if change_pct < -self.threshold {
                        msg += "Signal: *Sell 🎯*";
                    } else {
                        msg += "Signal: *Hold 🚫*";
                    }
                    println!("{}", msg);
                    tg.send(&msg);
                }
                Err(e) => eprintln!("Error fetching klines: {:?}", e),
            }
        }
        Ok(())
    }

    // Helper function to prepare the last row of features for prediction.
    // fn prepare_last_features(
    //     &self,
    //     df: DataFrame,
    //     feature_cols: &[String],
    // ) -> PolarsResult<DenseMatrix<f64>> {
    //     let df_last = df.tail(Some(1));
    //     let X_last = df_last.select(feature_cols)?.to_ndarray::<Float64Type>()?;
    //     Ok(DenseMatrix::from_2d_array(&X_last))
    // }
}
