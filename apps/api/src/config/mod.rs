use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ConfigError {
    MissingEnv(String),
    InvalidLength(&'static str, usize, usize),
    InvalidFormat(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnv(key) => write!(f, "Missing environment variable: {}", key),
            Self::InvalidLength(key, expected, actual) => write!(
                f,
                "Invalid length for {}: expected {}, got {}",
                key, expected, actual
            ),
            Self::InvalidFormat(key) => write!(f, "Invalid format for {}", key),
        }
    }
}
impl std::error::Error for ConfigError {}

#[derive(Clone, Debug)]
pub struct Config {
    pub mongo_uri: String,
    pub jwt_secret: String,
    pub port: u16,
    pub domain: String,
    pub storage_destination: PathBuf,
    pub storage_master_key: [u8; 32],
    pub max_upload_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mongo_uri =
            env::var("MONGO_URI").map_err(|_| ConfigError::MissingEnv("MONGO_URI".into()))?;

        let jwt_secret = env::var("JWT_SECRET_KEY")
            .map_err(|_| ConfigError::MissingEnv("JWT_SECRET_KEY".into()))?;
        if jwt_secret.len() < 32 {
            return Err(ConfigError::InvalidLength(
                "JWT_SECRET_KEY",
                32,
                jwt_secret.len(),
            ));
        }

        let storage_master_key_hex = env::var("STORAGE_MASTER_KEY_HEX")
            .map_err(|_| ConfigError::MissingEnv("STORAGE_MASTER_KEY_HEX".into()))?;
        if storage_master_key_hex.len() != 64 {
            return Err(ConfigError::InvalidLength(
                "STORAGE_MASTER_KEY_HEX",
                64,
                storage_master_key_hex.len(),
            ));
        }

        let mut storage_master_key = [0u8; 32];
        hex::decode_to_slice(&storage_master_key_hex, &mut storage_master_key)
            .map_err(|_| ConfigError::InvalidFormat("STORAGE_MASTER_KEY_HEX"))?;

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3821".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidFormat("PORT"))?;

        let domain =
            env::var("DRIVE_DOMAIN").unwrap_or_else(|_| format!("http://localhost:{}", port));

        let storage_destination = env::var("STORAGE_DESTINATION")
            .unwrap_or_else(|_| "bucket/storage".to_string())
            .into();

        let max_upload_bytes = env::var("MAX_UPLOAD_BYTES")
            .unwrap_or_else(|_| "104857600".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidFormat("MAX_UPLOAD_BYTES"))?;
        if max_upload_bytes == 0 {
            return Err(ConfigError::InvalidFormat("MAX_UPLOAD_BYTES"));
        }

        Ok(Self {
            mongo_uri,
            jwt_secret,
            port,
            domain,
            storage_destination,
            storage_master_key,
            max_upload_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn reset_env() {
        unsafe {
            env::remove_var("MONGO_URI");
            env::remove_var("JWT_SECRET_KEY");
            env::remove_var("STORAGE_MASTER_KEY_HEX");
            env::remove_var("PORT");
            env::remove_var("DRIVE_DOMAIN");
            env::remove_var("STORAGE_DESTINATION");
            env::remove_var("MAX_UPLOAD_BYTES");
        }
    }

    #[test]
    #[serial]
    fn missing_mongo_uri_fails() {
        reset_env();
        unsafe {
            env::set_var("JWT_SECRET_KEY", "01234567890123456789012345678901");
            env::set_var(
                "STORAGE_MASTER_KEY_HEX",
                "0000000000000000000000000000000000000000000000000000000000000000",
            );
        }
        assert!(matches!(Config::from_env(), Err(ConfigError::MissingEnv(e)) if e == "MONGO_URI"));
    }

    #[test]
    #[serial]
    fn missing_jwt_secret_fails() {
        reset_env();
        unsafe {
            env::set_var("MONGO_URI", "mongodb://localhost:27017");
            env::set_var(
                "STORAGE_MASTER_KEY_HEX",
                "0000000000000000000000000000000000000000000000000000000000000000",
            );
        }
        assert!(
            matches!(Config::from_env(), Err(ConfigError::MissingEnv(e)) if e == "JWT_SECRET_KEY")
        );
    }

    #[test]
    #[serial]
    fn short_jwt_secret_fails() {
        reset_env();
        unsafe {
            env::set_var("MONGO_URI", "mongodb://localhost:27017");
            env::set_var("JWT_SECRET_KEY", "short");
            env::set_var(
                "STORAGE_MASTER_KEY_HEX",
                "0000000000000000000000000000000000000000000000000000000000000000",
            );
        }
        assert!(
            matches!(Config::from_env(), Err(ConfigError::InvalidLength(e, _, _)) if e == "JWT_SECRET_KEY")
        );
    }

    #[test]
    #[serial]
    fn missing_storage_master_key_fails() {
        reset_env();
        unsafe {
            env::set_var("MONGO_URI", "mongodb://localhost:27017");
            env::set_var("JWT_SECRET_KEY", "01234567890123456789012345678901");
        }
        assert!(
            matches!(Config::from_env(), Err(ConfigError::MissingEnv(e)) if e == "STORAGE_MASTER_KEY_HEX")
        );
    }

    #[test]
    #[serial]
    fn invalid_storage_master_key_hex_fails() {
        reset_env();
        unsafe {
            env::set_var("MONGO_URI", "mongodb://localhost:27017");
            env::set_var("JWT_SECRET_KEY", "01234567890123456789012345678901");
            env::set_var("STORAGE_MASTER_KEY_HEX", "invalidhex");
        }
        assert!(
            matches!(Config::from_env(), Err(ConfigError::InvalidLength(e, _, _)) if e == "STORAGE_MASTER_KEY_HEX")
        );

        unsafe {
            env::set_var(
                "STORAGE_MASTER_KEY_HEX",
                "Z000000000000000000000000000000000000000000000000000000000000000",
            );
        }
        assert!(
            matches!(Config::from_env(), Err(ConfigError::InvalidFormat(e)) if e == "STORAGE_MASTER_KEY_HEX")
        );
    }

    #[test]
    #[serial]
    fn valid_config_loads() {
        reset_env();
        unsafe {
            env::set_var("MONGO_URI", "mongodb://localhost:27017");
            env::set_var("JWT_SECRET_KEY", "01234567890123456789012345678901");
            env::set_var(
                "STORAGE_MASTER_KEY_HEX",
                "0000000000000000000000000000000000000000000000000000000000000000",
            );
        }

        let config = Config::from_env().unwrap();
        assert_eq!(config.mongo_uri, "mongodb://localhost:27017");
    }

    #[test]
    #[serial]
    fn invalid_optional_numbers_fail() {
        reset_env();
        unsafe {
            env::set_var("MONGO_URI", "mongodb://localhost:27017");
            env::set_var("JWT_SECRET_KEY", "01234567890123456789012345678901");
            env::set_var(
                "STORAGE_MASTER_KEY_HEX",
                "0000000000000000000000000000000000000000000000000000000000000000",
            );
            env::set_var("PORT", "not-a-port");
        }
        assert!(matches!(
            Config::from_env(),
            Err(ConfigError::InvalidFormat("PORT"))
        ));

        unsafe {
            env::remove_var("PORT");
            env::set_var("MAX_UPLOAD_BYTES", "0");
        }
        assert!(matches!(
            Config::from_env(),
            Err(ConfigError::InvalidFormat("MAX_UPLOAD_BYTES"))
        ));
    }
}
