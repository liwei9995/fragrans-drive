use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub mongo_uri: String,
    pub jwt_secret: String,
    pub port: u16,
    pub domain: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let mongo_uri = env::var("MONGO_URI")
            .unwrap_or_else(|_| "mongodb://127.0.1.1:27017/fragrans".to_string());

        let jwt_secret = env::var("JWT_SECRET_KEY")
            .unwrap_or_else(|_| "default_secret_please_change".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3821".to_string())
            .parse()
            .unwrap_or(3821);

        let domain =
            env::var("DRIVE_DOMAIN").unwrap_or_else(|_| format!("http://localhost:{}", port));

        Self {
            mongo_uri,
            jwt_secret,
            port,
            domain,
        }
    }
}
