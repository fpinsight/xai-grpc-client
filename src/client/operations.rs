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

        Ok(response.models.into_iter().map(Into::into).collect())
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
    pub async fn get_model(
        &mut self,
        name: impl Into<String>,
    ) -> Result<crate::models::LanguageModel> {
        let request = proto::GetModelRequest { name: name.into() };

        let response = self
            .models_client
            .get_language_model(request)
            .await?
            .into_inner();

        Ok(response.into())
    }

    /// List all available embedding models
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///     let models = client.list_embedding_models().await?;
    ///
    ///     for model in models {
    ///         println!("{}: {}", model.name, model.version);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn list_embedding_models(&mut self) -> Result<Vec<crate::models::EmbeddingModel>> {
        let response = self
            .models_client
            .list_embedding_models(())
            .await?
            .into_inner();

        Ok(response.models.into_iter().map(Into::into).collect())
    }

    /// Get detailed information about a specific embedding model by name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///     let model = client.get_embedding_model("embed-large-v1").await?;
    ///
    ///     println!("Model: {}", model.name);
    ///     println!("Supports multimodal: {}",
    ///         model.input_modalities.len() > 1);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_embedding_model(
        &mut self,
        name: impl Into<String>,
    ) -> Result<crate::models::EmbeddingModel> {
        let request = proto::GetModelRequest { name: name.into() };

        let response = self
            .models_client
            .get_embedding_model(request)
            .await?
            .into_inner();

        Ok(response.into())
    }

    /// List all available image generation models
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///     let models = client.list_image_generation_models().await?;
    ///
    ///     for model in models {
    ///         println!("{}: ${:.2} per image",
    ///             model.name, model.image_price as f64 / 100.0);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn list_image_generation_models(
        &mut self,
    ) -> Result<Vec<crate::models::ImageGenerationModel>> {
        let response = self
            .models_client
            .list_image_generation_models(())
            .await?
            .into_inner();

        Ok(response.models.into_iter().map(Into::into).collect())
    }

    /// Get detailed information about a specific image generation model by name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///     let model = client.get_image_generation_model("image-gen-1").await?;
    ///
    ///     println!("Model: {}", model.name);
    ///     println!("Max prompt length: {}", model.max_prompt_length);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_image_generation_model(
        &mut self,
        name: impl Into<String>,
    ) -> Result<crate::models::ImageGenerationModel> {
        let request = proto::GetModelRequest { name: name.into() };

        let response = self
            .models_client
            .get_image_generation_model(request)
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
    pub async fn embed(
        &mut self,
        request: crate::embedding::EmbedRequest,
    ) -> Result<crate::embedding::EmbedResponse> {
        let proto_request = self.embed_request_to_proto(&request);

        let response = self
            .embedder_client
            .embed(proto_request)
            .await?
            .into_inner();

        Self::proto_to_embed_response(response)
    }

    /// Tokenize text to count tokens and understand token boundaries.
    ///
    /// This is useful for:
    /// - Estimating costs before making requests
    /// - Understanding how text is split into tokens
    /// - Staying within model token limits
    /// - Debugging prompt construction
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::{GrokClient, TokenizeRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///
    ///     let request = TokenizeRequest::new("grok-2-1212")
    ///         .with_text("Hello, world! How are you today?");
    ///
    ///     let response = client.tokenize(request).await?;
    ///     println!("Token count: {}", response.token_count());
    ///
    ///     for token in &response.tokens {
    ///         println!("Token: '{}' (ID: {})", token.string_token, token.token_id);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn tokenize(
        &mut self,
        request: crate::tokenize::TokenizeRequest,
    ) -> Result<crate::tokenize::TokenizeResponse> {
        let proto_request = proto::TokenizeTextRequest {
            text: request.text,
            model: request.model,
            user: request.user.unwrap_or_default(),
        };

        let response = self
            .tokenize_client
            .tokenize_text(proto_request)
            .await?
            .into_inner();

        let tokens = response
            .tokens
            .into_iter()
            .map(|t| crate::tokenize::Token {
                token_id: t.token_id,
                string_token: t.string_token,
                token_bytes: t.token_bytes,
            })
            .collect();

        Ok(crate::tokenize::TokenizeResponse {
            tokens,
            model: response.model,
        })
    }

    /// Get information about the current API key.
    ///
    /// This method returns metadata about your API key including:
    /// - Redacted key value
    /// - User and team IDs
    /// - Permissions (ACLs)
    /// - Status (blocked, disabled, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xai_grpc_client::GrokClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = GrokClient::from_env().await?;
    ///     let api_key_info = client.get_api_key_info().await?;
    ///
    ///     println!("API Key: {}", api_key_info.redacted_api_key);
    ///     println!("Team ID: {}", api_key_info.team_id);
    ///     println!("Status: {}", api_key_info.status_string());
    ///
    ///     if !api_key_info.is_active() {
    ///         println!("Warning: API key is not active!");
    ///     }
    ///
    ///     println!("Permissions:");
    ///     for acl in &api_key_info.acls {
    ///         println!("  - {}", acl);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_api_key_info(&mut self) -> Result<crate::api_key::ApiKeyInfo> {
        let response = self.auth_client.get_api_key_info(()).await?.into_inner();

        Ok(response.into())
    }

    /// Sample text using the Sample API (alternative to Chat API).
    ///
    /// This is a simpler API for basic text completion without conversation structure.
    /// For most use cases, `complete_chat()` is recommended.
    pub async fn sample_text(
        &mut self,
        request: crate::sample::SampleRequest,
    ) -> Result<crate::sample::SampleResponse> {
        let proto_request = proto::SampleTextRequest {
            prompt: request.prompts,
            model: request.model,
            n: request.n,
            max_tokens: request.max_tokens,
            seed: request.seed,
            stop: request.stop,
            temperature: request.temperature,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            logprobs: request.logprobs,
            presence_penalty: request.presence_penalty,
            top_logprobs: request.top_logprobs,
            user: request.user.unwrap_or_default(),
        };

        let response = self
            .sample_client
            .sample_text(proto_request)
            .await?
            .into_inner();

        Ok(response.into())
    }

    /// Stream text sampling (alternative to streaming chat).
    pub async fn sample_text_streaming(
        &mut self,
        request: crate::sample::SampleRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<crate::sample::SampleResponse>> + Send>>> {
        let proto_request = proto::SampleTextRequest {
            prompt: request.prompts,
            model: request.model,
            n: request.n,
            max_tokens: request.max_tokens,
            seed: request.seed,
            stop: request.stop,
            temperature: request.temperature,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            logprobs: request.logprobs,
            presence_penalty: request.presence_penalty,
            top_logprobs: request.top_logprobs,
            user: request.user.unwrap_or_default(),
        };

        let response = self
            .sample_client
            .sample_text_streaming(proto_request)
            .await?
            .into_inner();

        let stream = response.map(|result| result.map_err(Into::into).map(Into::into));

        Ok(Box::pin(stream))
    }

    /// Generate images from text prompts.
    pub async fn generate_image(
        &mut self,
        request: crate::image::ImageGenerationRequest,
    ) -> Result<crate::image::ImageGenerationResponse> {
        let proto_request = proto::GenerateImageRequest {
            prompt: request.prompt,
            image: request.image_url.map(|url| proto::ImageUrlContent {
                image_url: url,
                detail: proto::ImageDetail::DetailAuto as i32,
            }),
            model: request.model,
            n: request.n,
            user: request.user.unwrap_or_default(),
            format: match request.format {
                crate::image::ImageFormat::Base64 => proto::ImageFormat::ImgFormatBase64 as i32,
                crate::image::ImageFormat::Url => proto::ImageFormat::ImgFormatUrl as i32,
            },
        };

        let response = self
            .image_client
            .generate_image(proto_request)
            .await?
            .into_inner();

        Ok(response.into())
    }

    /// Search documents in collections for RAG applications.
    pub async fn search_documents(
        &mut self,
        request: crate::documents::DocumentSearchRequest,
    ) -> Result<crate::documents::DocumentSearchResponse> {
        let proto_request = proto::SearchRequest {
            query: request.query,
            source: Some(proto::DocumentsSource {
                collection_ids: request.collection_ids,
            }),
            limit: request.limit,
            ranking_metric: Some(match request.ranking_metric {
                crate::documents::RankingMetric::L2Distance => {
                    proto::RankingMetric::L2Distance as i32
                }
                crate::documents::RankingMetric::CosineSimilarity => {
                    proto::RankingMetric::CosineSimilarity as i32
                }
            }),
            instructions: request.instructions,
        };

        let response = self
            .documents_client
            .search(proto_request)
            .await?
            .into_inner();

        Ok(response.into())
    }
}
