use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::*;

pub const CONFIG_FILE: &str = ".config.toml";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub credentials: Credentials,
    pub settings: Settings,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub account_id: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub bucket: String,
}

impl Config {
    pub fn default(bucket: Option<String>) -> String {
        let config = Self {
            credentials: Credentials {
                account_id: "<your-cloudflare-account-id>".into(),
                access_key_id: "<your-r2-access-key-id>".into(),
                secret_access_key: "<your-r2-secret-access-key>".into(),
            },
            settings: Settings {
                bucket: bucket.unwrap_or("<your-bucket>".into()),
            },
        };

        toml::to_string_pretty(&config).unwrap()
    }

    pub fn load() -> RsyncResult<Self> {
        let path = Path::new(CONFIG_FILE);

        if !path.exists() {
            return Err(Error::Persistance(format!(
                "no {CONFIG_FILE} was found in this directory"
            )));
        }

        let raw = std::fs::read_to_string(path)
            .map_err(|err| Error::Persistance(format!("cannot read {CONFIG_FILE}: {err}")))?;

        let config: Config = toml::from_str(&raw)
            .map_err(|err| Error::Persistance(format!("cannot parse {CONFIG_FILE}: {err}")))?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> RsyncResult<()> {
        let fields = [
            ("account_id", &self.credentials.account_id),
            ("access_key_id", &self.credentials.access_key_id),
            ("secret_access_key", &self.credentials.secret_access_key),
            ("bucket", &self.settings.bucket),
        ];

        for (name, value) in fields {
            if value.trim().is_empty() || value.contains('<') {
                return Err(Error::Persistance(format!(
                    "invalid {CONFIG_FILE}: '{name}' is not configured properly"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_config() {
        let config: Config = toml::from_str(
            r#"
            [credentials]
            account_id = "abc123"
            access_key_id = "key"
            secret_access_key = "secret"

            [settings]
            bucket = "my-bucket"
            "#,
        )
        .unwrap();

        assert_eq!(config.credentials.account_id, "abc123");
        assert_eq!(config.credentials.access_key_id, "key");
        assert_eq!(config.credentials.secret_access_key, "secret");
        assert_eq!(config.settings.bucket, "my-bucket");
        assert!(config.validate().is_ok())
    }

    #[test]
    fn invalid_config() {
        let config_raw = Config::default(None);
        let config: Config = toml::from_str(&config_raw).unwrap();

        assert!(config.validate().is_err())
    }
}
