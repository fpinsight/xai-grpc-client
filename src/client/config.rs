use crate::{
    auth::AuthInterceptor,
    error::{GrokError, Result},
    proto::chat_client::ChatClient,
    proto::models_client::ModelsClient,
};
use secrecy::{ExposeSecret, SecretString};
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

/// Configuration for the Grok API client.
///
/// This struct contains all the settings needed to connect to the xAI Grok API,
/// including the endpoint URL, API key, default model, and timeout settings.
///
/// # Examples
///
/// ```no_run
/// use xai_grpc_client::GrokConfig;
/// use secrecy::SecretString;
/// use std::time::Duration;
///
/// let config = GrokConfig {
///     endpoint: "https://api.x.ai".to_string(),
///     api_key: SecretString::from("your-api-key".to_string()),
///     default_model: "grok-2-1212".to_string(),
///     timeout: Duration::from_secs(120),
/// };
/// ```
#[derive(Clone)]
pub struct GrokConfig {
    /// The gRPC endpoint URL (default: <https://api.x.ai>).
    pub endpoint: String,

    /// API key for authentication (stored securely using SecretString).
    pub api_key: SecretString,

    /// Default model to use for requests (default: "grok-code-fast-1").
    pub default_model: String,

    /// Request timeout duration (default: 60 seconds).
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

/// The main client for interacting with the xAI Grok API.
///
/// `GrokClient` provides methods for chat completions, streaming responses,
/// deferred completions, and managing stored completions.
///
/// # Examples
///
/// ## Creating a client from environment
///
/// ```no_run
/// use xai_grpc_client::GrokClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Reads XAI_API_KEY from environment
/// let mut client = GrokClient::from_env().await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Creating a client with custom configuration
///
/// ```no_run
/// use xai_grpc_client::{GrokClient, GrokConfig};
/// use secrecy::SecretString;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = GrokConfig {
///     endpoint: "https://api.x.ai".to_string(),
///     api_key: SecretString::from("your-api-key".to_string()),
///     default_model: "grok-2-1212".to_string(),
///     timeout: Duration::from_secs(120),
/// };
///
/// let mut client = GrokClient::new(config).await?;
/// # Ok(())
/// # }
/// ```
pub struct GrokClient {
    pub(super) inner:
        ChatClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) models_client:
        ModelsClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) config: GrokConfig,
}

impl GrokClient {
    /// Creates a client using the `XAI_API_KEY` environment variable.
    ///
    /// This is the simplest way to create a client. It uses default settings
    /// for endpoint, model, and timeout.
    ///
    /// # Environment Variables
    ///
    /// - `XAI_API_KEY` - Your xAI API key (required)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `XAI_API_KEY` environment variable is not set
    /// - The API key is invalid
    /// - Connection to the API fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = GrokClient::from_env().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_env() -> Result<Self> {
        let api_key = std::env::var("XAI_API_KEY")?;

        let config = GrokConfig {
            api_key: SecretString::from(api_key),
            ..Default::default()
        };

        Self::new(config).await
    }

    /// Creates a client with custom configuration.
    ///
    /// Use this method when you need to customize the endpoint, model,
    /// timeout, or provide the API key programmatically.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the client
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - API key is empty
    /// - Endpoint URL is invalid
    /// - Connection to the API fails
    /// - TLS configuration fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::{GrokClient, GrokConfig};
    /// use secrecy::SecretString;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = GrokConfig {
    ///     endpoint: "https://api.x.ai".to_string(),
    ///     api_key: SecretString::from("your-api-key".to_string()),
    ///     default_model: "grok-2-1212".to_string(),
    ///     timeout: Duration::from_secs(120),
    /// };
    ///
    /// let mut client = GrokClient::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
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
        let inner = ChatClient::with_interceptor(channel.clone(), interceptor.clone());
        let models_client = ModelsClient::with_interceptor(channel, interceptor);

        Ok(Self { inner, models_client, config })
    }

    /// Tests the connection by sending a simple request to the API.
    ///
    /// This method verifies that the client is properly configured and can
    /// communicate with the xAI API. It sends a simple "Hello" message and
    /// returns the response.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Connection to the API fails
    /// - Authentication fails
    /// - The API returns an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = GrokClient::from_env().await?;
    /// let response = client.test_connection().await?;
    /// println!("Connection test: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn test_connection(&mut self) -> Result<String> {
        use crate::proto::{content, Content, GetCompletionsRequest, Message, MessageRole};

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

        Err(GrokError::InvalidRequest(
            "No content in response".to_string(),
        ))
    }
}
