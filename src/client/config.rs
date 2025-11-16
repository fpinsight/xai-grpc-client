use std::time::Duration;
use secrecy::{ExposeSecret, SecretString};
use tonic::transport::{Channel, Endpoint};
use crate::{
    auth::AuthInterceptor,
    error::{Result, GrokError},
    proto::chat_client::ChatClient,
};

#[derive(Clone)]
pub struct GrokConfig {
    pub endpoint: String,
    pub api_key: SecretString,
    pub default_model: String,
    pub timeout: Duration,
}

impl Default for GrokConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.x.ai".to_string(),
            api_key: SecretString::from(String::new()),
            default_model: "grok-code-fast-1".to_string(),
            timeout: Duration::from_secs(60),
        }
    }
}

pub struct GrokClient {
    pub(super) inner: ChatClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) config: GrokConfig,
}

impl GrokClient {
    /// Create client from XAI_API_KEY environment variable
    pub async fn from_env() -> Result<Self> {
        let api_key = std::env::var("XAI_API_KEY")?;

        let config = GrokConfig {
            api_key: SecretString::from(api_key),
            ..Default::default()
        };

        Self::new(config).await
    }

    /// Create client with explicit configuration
    pub async fn new(config: GrokConfig) -> Result<Self> {
        if config.api_key.expose_secret().is_empty() {
            return Err(GrokError::Config("API key is empty".to_string()));
        }

        // Configure TLS for HTTPS (tls-webpki-roots feature provides root certs)
        let tls_config = tonic::transport::ClientTlsConfig::new();

        let endpoint = Endpoint::from_shared(config.endpoint.clone())?
            .timeout(config.timeout)
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(10))
            .tls_config(tls_config)?;

        let channel = endpoint.connect().await?;

        let interceptor = AuthInterceptor::new(config.api_key.clone());
        let inner = ChatClient::with_interceptor(channel, interceptor);

        Ok(Self { inner, config })
    }

    /// Simple test method - sends "Hello" and returns response
    pub async fn test_connection(&mut self) -> Result<String> {
        use crate::proto::{MessageRole, Content, content, Message, GetCompletionsRequest};

        let request = GetCompletionsRequest {
            messages: vec![Message {
                role: MessageRole::RoleUser as i32,
                content: vec![Content {
                    content: Some(content::Content::Text(
                        "Say 'Hello from gRPC!' in one sentence.".to_string(),
                    )),
                }],
                ..Default::default()
            }],
            model: self.config.default_model.clone(),
            max_tokens: Some(50),
            ..Default::default()
        };

        let response = self.inner.get_completion(request).await?;
        let completion = response.into_inner();

        // Extract text from first output
        if let Some(output) = completion.outputs.first() {
            if let Some(message) = &output.message {
                return Ok(message.content.clone());
            }
        }

        Err(GrokError::InvalidRequest("No content in response".to_string()))
    }
}
