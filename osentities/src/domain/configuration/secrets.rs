use envconfig::Envconfig;
use std::fmt::{Display, Formatter, Result};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum SecretServiceProvider {
    GoogleKms,
    IosKms,
}

#[derive(Debug, Clone, Envconfig)]
pub struct SecretsConfig {
    #[envconfig(from = "SECRETS_SERVICE_PROVIDER", default = "google-kms")]
    pub provider: SecretServiceProvider,
    #[envconfig(from = "GOOGLE_KMS_PROJECT_ID", default = "buildable-production")]
    pub google_kms_project_id: String,
    #[envconfig(from = "GOOGLE_KMS_LOCATION_ID", default = "global")]
    pub google_kms_location_id: String,
    #[envconfig(
        from = "GOOGLE_KMS_KEY_RING_ID",
        default = "secrets-service-development"
    )]
    pub google_kms_key_ring_id: String,
    #[envconfig(from = "GOOGLE_KMS_KEY_ID", default = "secrets-service-development")]
    pub google_kms_key_id: String,
    #[envconfig(
        from = "IOS_CRYPTO_SECRET",
        default = "xTtUQejH8eSNmWP5rlnHLkOWkHeflivG"
    )]
    pub ios_crypto_secret: String,
}

impl SecretsConfig {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn with_secret(mut self, secret: String) -> Self {
        self.ios_crypto_secret = secret;
        self
    }

    #[cfg(test)]
    pub fn with_provider(mut self, provider: SecretServiceProvider) -> Self {
        self.provider = provider;
        self
    }
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            provider: SecretServiceProvider::GoogleKms,
            google_kms_project_id: "buildable-production".to_owned(),
            google_kms_location_id: "global".to_owned(),
            google_kms_key_ring_id: "secrets-service-local".to_owned(),
            google_kms_key_id: "secrets-service-local".to_owned(),
            ios_crypto_secret: "xTtUQejH8eSNmWP5rlnHLkOWkHeflivG".to_owned(),
        }
    }
}

impl Display for SecretsConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "SECRETS_SERVICE_PROVIDER: {}", self.provider.as_ref())?;
        match self.provider {
            SecretServiceProvider::GoogleKms => {
                writeln!(f, "GOOGLE_KMS_PROJECT_ID: ****")?;
                writeln!(f, "GOOGLE_KMS_LOCATION_ID: ****")?;
                writeln!(f, "GOOGLE_KMS_KEY_RING_ID: ****")?;
                writeln!(f, "GOOGLE_KMS_KEY_ID: ****")
            }
            SecretServiceProvider::IosKms => writeln!(f, "IOS_CRYPTO_SECRET: ****"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config() {
        let config = SecretsConfig::new();

        assert_eq!(
            config.ios_crypto_secret.as_str(),
            "xTtUQejH8eSNmWP5rlnHLkOWkHeflivG"
        );
        assert_eq!(config.provider, SecretServiceProvider::GoogleKms);
        assert_eq!(config.google_kms_project_id, "buildable-production");
        assert_eq!(config.google_kms_location_id, "global");
        assert_eq!(config.google_kms_key_ring_id, "secrets-service-local");
        assert_eq!(config.google_kms_key_id, "secrets-service-local");
    }

    #[tokio::test]
    async fn test_config_display() {
        let config = SecretsConfig::new();

        let config_str = format!("{config}");

        let display = "SECRETS_SERVICE_PROVIDER: google-kms\n\
            GOOGLE_KMS_PROJECT_ID: ****\n\
            GOOGLE_KMS_LOCATION_ID: ****\n\
            GOOGLE_KMS_KEY_RING_ID: ****\n\
            GOOGLE_KMS_KEY_ID: ****\n\
            ";

        assert_eq!(config_str, display);
    }
}
