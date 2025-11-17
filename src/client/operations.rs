use super::config::GrokClient;
use crate::{
    error::{GrokError, Result},
    proto,
    request::ChatRequest,
    response::{ChatChunk, ChatResponse},
};
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt};

impl GrokClient {
    /// Blocking completion (for simple queries)
    pub async fn complete_chat(&mut self, request: ChatRequest) -> Result<ChatResponse> {
        let proto_request = self.to_proto_request(&request)?;

        let response = self.inner.get_completion(proto_request).await?.into_inner();

        self.proto_to_response(response)
    }

    /// Stream chat completion (PRIMARY for REPL)
    pub async fn stream_chat(
        &mut self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk>> + Send>>> {
        let proto_request = self.to_proto_request(&request)?;

        let response = self
            .inner
            .get_completion_chunk(proto_request)
            .await?
            .into_inner();

        let stream = response.map(|result| {
            result
                .map_err(Into::into)
                .and_then(Self::proto_chunk_to_chunk)
        });

        Ok(Box::pin(stream))
    }

    /// Start a deferred completion (async polling mode)
    /// Returns a request_id that can be used to poll for results
    pub async fn start_deferred(&mut self, request: ChatRequest) -> Result<String> {
        let proto_request = self.to_proto_request(&request)?;

        let response = self
            .inner
            .start_deferred_completion(proto_request)
            .await?
            .into_inner();

        Ok(response.request_id)
    }

    /// Poll for deferred completion results
    /// Returns None if still pending, Some(response) if complete
    pub async fn poll_deferred(&mut self, request_id: String) -> Result<Option<ChatResponse>> {
        let proto_request = proto::GetDeferredRequest { request_id };

        let response = self
            .inner
            .get_deferred_completion(proto_request)
            .await?
            .into_inner();

        // Check status
        let status = proto::DeferredStatus::try_from(response.status)
            .unwrap_or(proto::DeferredStatus::InvalidDeferredStatus);

        match status {
            proto::DeferredStatus::Done => {
                // Response is ready
                if let Some(completion_response) = response.response {
                    Ok(Some(self.proto_to_response(completion_response)?))
                } else {
                    Err(GrokError::InvalidRequest(
                        "Deferred request marked as done but no response".to_string(),
                    ))
                }
            }
            proto::DeferredStatus::Pending => {
                // Still processing
                Ok(None)
            }
            proto::DeferredStatus::Expired => Err(GrokError::InvalidRequest(
                "Deferred request has expired".to_string(),
            )),
            proto::DeferredStatus::InvalidDeferredStatus => Err(GrokError::InvalidRequest(
                "Invalid deferred status".to_string(),
            )),
        }
    }

    /// Wait for deferred completion to finish (blocking with polling)
    /// Polls every `poll_interval` until complete or timeout
    pub async fn wait_for_deferred(
        &mut self,
        request_id: String,
        poll_interval: std::time::Duration,
        timeout: std::time::Duration,
    ) -> Result<ChatResponse> {
        use tokio::time::{sleep, Instant};

        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(GrokError::InvalidRequest(
                    "Deferred request timed out".to_string(),
                ));
            }

            match self.poll_deferred(request_id.clone()).await? {
                Some(response) => return Ok(response),
                None => sleep(poll_interval).await,
            }
        }
    }

    /// Retrieve a stored completion by response ID
    /// Used when store_messages was set to true in the original request
    pub async fn get_stored_completion(&mut self, response_id: String) -> Result<ChatResponse> {
        let proto_request = proto::GetStoredCompletionRequest { response_id };

        let response = self
            .inner
            .get_stored_completion(proto_request)
            .await?
            .into_inner();

        self.proto_to_response(response)
    }

    /// Delete a stored completion by response ID
    pub async fn delete_stored_completion(&mut self, response_id: String) -> Result<()> {
        let proto_request = proto::DeleteStoredCompletionRequest { response_id };

        self.inner.delete_stored_completion(proto_request).await?;

        Ok(())
    }

    /// List all available language models
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///     let models = client.list_models().await?;
    ///
    ///     for model in models {
    ///         println!("{}: {} (max {} tokens)",
    ///             model.name, model.version, model.max_prompt_length);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn list_models(&mut self) -> Result<Vec<crate::models::LanguageModel>> {
        let response = self
            .models_client
            .list_language_models(())
            .await?
            .into_inner();

        Ok(response
            .models
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Get detailed information about a specific model by name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///     let model = client.get_model("grok-2-1212").await?;
    ///
    ///     println!("Model: {}", model.name);
    ///     println!("Context length: {}", model.max_prompt_length);
    ///     println!("Prompt price: ${:.4}/1M tokens",
    ///         model.prompt_text_token_price as f64 / 100.0 / 1_000_000.0);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_model(&mut self, name: impl Into<String>) -> Result<crate::models::LanguageModel> {
        let request = proto::GetModelRequest { name: name.into() };

        let response = self
            .models_client
            .get_language_model(request)
            .await?
            .into_inner();

        Ok(response.into())
    }

    /// Generate embeddings from text or images.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::{GrokClient, EmbedRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///
    ///     let request = EmbedRequest::new("embed-large-v1")
    ///         .add_text("Hello, world!")
    ///         .add_text("How are you?");
    ///
    ///     let response = client.embed(request).await?;
    ///
    ///     for embedding in response.embeddings {
    ///         println!("Embedding {} has {} dimensions",
    ///             embedding.index, embedding.vector.len());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn embed(&mut self, request: crate::embedding::EmbedRequest) -> Result<crate::embedding::EmbedResponse> {
        let proto_request = self.embed_request_to_proto(&request);

        let response = self
            .embedder_client
            .embed(proto_request)
            .await?
            .into_inner();

        Self::proto_to_embed_response(response)
    }
}
