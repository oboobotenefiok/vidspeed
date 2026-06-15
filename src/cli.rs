use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub temp_file_ttl_hours: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(Config {
            temp_file_ttl_hours: std::env::var("TEMP_FILE_TTL_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()?,
        })
    }
}
