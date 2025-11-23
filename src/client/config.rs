use crate::{
    auth::AuthInterceptor,
    error::{GrokError, Result},
    proto::auth_client::AuthClient,
    proto::chat_client::ChatClient,
    proto::documents_client::DocumentsClient,
    proto::embedder_client::EmbedderClient,
    proto::image_client::ImageClient,
    proto::models_client::ModelsClient,
    proto::sample_client::SampleClient,
    proto::tokenize_client::TokenizeClient,
};
use secrecy::{ExposeSecret, SecretString};
use std::time::Duration;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use url::Url;

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
    pub(super) embedder_client:
        EmbedderClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) tokenize_client:
        TokenizeClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) auth_client:
        AuthClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) sample_client:
        SampleClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) image_client:
        ImageClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) documents_client:
        DocumentsClient<tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>>,
    pub(super) config: GrokConfig,
}

impl GrokClient {
    /// Creates a client with a custom configured channel.
    ///
    /// This constructor provides maximum flexibility by allowing you to bring
    /// your own configured `Channel`. Use this when you need:
    ///
    /// - **Custom TLS configuration** (e.g., custom CA certificates, specific domain validation)
    /// - **Proxy support** through custom channel configuration
    /// - **Custom middleware** or tracing layers
    /// - **Mock channels** for testing
    /// - **Custom timeouts** or connection pooling
    ///
    /// # Arguments
    ///
    /// * `channel` - A configured tonic Channel
    /// * `api_key` - Your xAI API key (stored securely using SecretString)
    ///
    /// # Examples
    ///
    /// ## Custom TLS with specific domain validation
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    /// use tonic::transport::{Channel, ClientTlsConfig};
    /// use secrecy::SecretString;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tls_config = ClientTlsConfig::new()
    ///     .domain_name("api.x.ai");
    ///
    /// let channel = Channel::from_static("https://api.x.ai")
    ///     .timeout(Duration::from_secs(120))
    ///     .tls_config(tls_config)?
    ///     .connect()
    ///     .await?;
    ///
    /// let api_key = SecretString::from("xai-your-key".to_string());
    /// let client = GrokClient::with_channel(channel, api_key);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Custom CA certificate
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    /// use tonic::transport::{Channel, ClientTlsConfig, Certificate};
    /// use secrecy::SecretString;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ca_cert = std::fs::read("path/to/ca.pem")?;
    /// let ca = Certificate::from_pem(ca_cert);
    ///
    /// let tls_config = ClientTlsConfig::new()
    ///     .ca_certificate(ca)
    ///     .domain_name("api.x.ai");
    ///
    /// let channel = Channel::from_static("https://api.x.ai")
    ///     .tls_config(tls_config)?
    ///     .connect()
    ///     .await?;
    ///
    /// let api_key = SecretString::from("xai-your-key".to_string());
    /// let client = GrokClient::with_channel(channel, api_key);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_channel(channel: Channel, api_key: SecretString) -> Self {
        let interceptor = AuthInterceptor::new(api_key.clone());

        let inner = ChatClient::with_interceptor(channel.clone(), interceptor.clone());
        let models_client = ModelsClient::with_interceptor(channel.clone(), interceptor.clone());
        let embedder_client =
            EmbedderClient::with_interceptor(channel.clone(), interceptor.clone());
        let tokenize_client =
            TokenizeClient::with_interceptor(channel.clone(), interceptor.clone());
        let auth_client = AuthClient::with_interceptor(channel.clone(), interceptor.clone());
        let sample_client = SampleClient::with_interceptor(channel.clone(), interceptor.clone());
        let image_client = ImageClient::with_interceptor(channel.clone(), interceptor.clone());
        let documents_client = DocumentsClient::with_interceptor(channel, interceptor);

        Self {
            inner,
            models_client,
            embedder_client,
            tokenize_client,
            auth_client,
            sample_client,
            image_client,
            documents_client,
            config: GrokConfig {
                endpoint: "https://api.x.ai".to_string(),
                api_key,
                default_model: "grok-code-fast-1".to_string(),
                timeout: Duration::from_secs(60),
            },
        }
    }

    /// Creates a client with default configuration using the provided API key.
    ///
    /// This is the simplest way to create a client when you have an API key
    /// at hand. It uses default settings for endpoint, model, and timeout.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your xAI API key
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Connection to the API fails
    /// - TLS configuration fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    /// use secrecy::SecretString;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api_key = SecretString::from("xai-your-key".to_string());
    /// let mut client = GrokClient::connect(api_key).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(api_key: SecretString) -> Result<Self> {
        let config = GrokConfig {
            api_key,
            ..Default::default()
        };

        Self::new(config).await
    }

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
    /// This method automatically configures TLS based on enabled features:
    /// - `tls-webpki-roots`: Uses Mozilla's root certificates (default)
    /// - `tls-native-roots`: Uses system native certificate store
    /// - Both features can be enabled simultaneously for fallback behavior
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

        // Build channel from config
        let channel = Self::build_channel_from_config(&config).await?;

        // Reuse with_channel logic
        let mut client = Self::with_channel(channel, config.api_key.clone());
        client.config = config; // Update config with provided values
        Ok(client)
    }

    /// Helper method to build a channel from GrokConfig.
    ///
    /// This method handles automatic TLS configuration based on enabled features
    /// and extracts the domain name from the endpoint URL for proper validation.
    async fn build_channel_from_config(config: &GrokConfig) -> Result<Channel> {
        // Parse the endpoint URL to extract the domain name for TLS validation
        let url = Url::parse(&config.endpoint)
            .map_err(|e| GrokError::Config(format!("Invalid endpoint URL: {e}")))?;
        let domain_name = url.host_str().ok_or_else(|| {
            GrokError::Config("Endpoint URL does not contain a valid host".to_string())
        })?;

        // Build TLS config with automatic root certificate selection
        let mut tls_config = ClientTlsConfig::new();

        // Note: If both features are enabled, both root stores will be used (fallback behavior)
        #[cfg(feature = "tls-webpki-roots")]
        {
            tls_config = tls_config.with_webpki_roots();
        }

        #[cfg(feature = "tls-native-roots")]
        {
            tls_config = tls_config.with_native_roots();
        }

        // Set domain name for TLS validation
        let tls_config = tls_config.domain_name(domain_name);

        // Build endpoint with optimized connection settings
        let endpoint = Endpoint::from_shared(config.endpoint.clone())?
            .timeout(config.timeout)
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(10))
            .tls_config(tls_config)?;

        // Connect and return channel
        endpoint.connect().await.map_err(Into::into)
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
