use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub max_file_size_mb: u64,
    pub max_concurrent_jobs: usize,
    pub temp_file_ttl_hours: u64,
    pub rate_limit_requests: u64,
    pub rate_limit_duration_secs: u64,
    pub redis_url: String,
    pub jwt_secret: String,
    pub allowed_origins: Vec<String>,
    pub upload_dir: String,
    pub processed_dir: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(Config {
            max_file_size_mb: std::env::var("MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "500".to_string())
                .parse()?,
            max_concurrent_jobs: std::env::var("MAX_CONCURRENT_JOBS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()?,
            temp_file_ttl_hours: std::env::var("TEMP_FILE_TTL_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()?,
            rate_limit_requests: std::env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            rate_limit_duration_secs: std::env::var("RATE_LIMIT_DURATION_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change_this_in_production".to_string()),
            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            upload_dir: std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),
            processed_dir: std::env::var("PROCESSED_DIR")
                .unwrap_or_else(|_| "./processed".to_string()),
        })
    }
}
