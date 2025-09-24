use reqwest::blocking::Client;

pub struct Telegram {
    client: Client,
    token: String,
    chat_id: String,
}

impl Telegram {
    /// Create a new Telegram client
    pub fn new() -> Self {
        let token = std::env::var("TELEGRAM_TOKEN").unwrap_or_default();
        let chat_id = std::env::var("TELEGRAM_TO").unwrap_or_default();
        if token.is_empty() {
            println!("[WARN] The TELEGRAM_TOKEN is missing in .env");
        }
        if chat_id.is_empty() {
            println!("[WARN] The TELEGRAM_TO is missing in .env");
        }

        Self {
            client: Client::new(),
            token,
            chat_id,
        }
    }

    /// Send a message (Markdown supported)
    pub fn send(&self, msg: &str) {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let resp = self
            .client
            .post(&url)
            .form(&[
                ("chat_id", self.chat_id.as_str()),
                ("text", msg),
                ("parse_mode", "Markdown"),
            ])
            .send();

        match resp {
            Ok(r) if r.status().is_success() => {
                println!("✅ [INFO] Telegram message sent")
            }
            Ok(r) => {
                println!("[WARN] Telegram failed: {}", r.status())
            }
            Err(e) => {
                println!("[ERROR] Telegram send failed: {}", e)
            }
        }
    }
}
