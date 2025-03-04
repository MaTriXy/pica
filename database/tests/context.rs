use database::algebra::init::DatabaseInitializer;
use database::algebra::init::Initializer;
use envconfig::Envconfig;
use http::{Method, StatusCode};
use osentities::prefix::IdPrefix;
use osentities::Id;
use osentities::{database::DatabasePodConfig, InternalError, PicaError};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::{collections::HashMap, sync::OnceLock, time::Duration};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{clients::Cli as Docker, Container},
};
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub static DOCKER: OnceLock<Docker> = OnceLock::new();
pub static POSTGRES: OnceLock<Container<'static, Postgres>> = OnceLock::new();
static TRACING: OnceLock<()> = OnceLock::new();

pub struct TestServer {
    pub port: u16,
    pub client: reqwest::Client,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ApiResponse<T: DeserializeOwned = Value> {
    pub code: StatusCode,
    pub data: T,
}

impl TestServer {
    pub async fn new(r#override: HashMap<String, String>) -> Result<Self, PicaError> {
        TRACING.get_or_init(|| {
            let filter = EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy();

            tracing_subscriber::fmt().with_env_filter(filter).init();
        });

        let server_port = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to port")
            .local_addr()
            .expect("Failed to get local address")
            .port();

        let config_map: HashMap<String, String> = HashMap::from([
            (
                "INTERNAL_SERVER_ADDRESS".to_string(),
                format!("0.0.0.0:{server_port}"),
            ),
            (
                "DATABASE_CONNECTION_TYPE".to_string(),
                "postgresql".to_string(),
            ),
            (
                "CONNECTION_ID".to_string(),
                Id::now(IdPrefix::Connection).to_string(),
            ),
            ("JWT_SECRET".to_string(), "secret".to_string()),
        ])
        .into_iter()
        .chain(r#override.into_iter())
        .collect::<HashMap<String, String>>();

        let config = DatabasePodConfig::init_from_hashmap(&config_map)
            .expect("Failed to initialize storage config");

        let server = DatabaseInitializer::init(&config).await?;

        tokio::task::spawn(async move { server.run().await });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        Ok(Self {
            port: server_port,
            client,
        })
    }

    pub async fn send_request<T: Serialize, U: DeserializeOwned + Debug>(
        &self,
        path: &str,
        method: Method,
        payload: Option<&T>,
    ) -> Result<ApiResponse<U>, PicaError> {
        let uri = format!("http://localhost:{}/{path}", self.port);
        let mut req = self.client.request(method, uri);
        if let Some(payload) = payload {
            req = req.json(payload);
        }

        let res = req
            .send()
            .await
            .map_err(|e| InternalError::io_err(&format!("Failed to send request: {}", e), None))?;

        let status = res.status();
        let json = res.json().await;

        Ok(ApiResponse {
            code: status,
            data: json.map_err(|e| {
                InternalError::deserialize_error(
                    &format!("Failed to deserialize response: {}", e),
                    None,
                )
            })?,
        })
    }
}
