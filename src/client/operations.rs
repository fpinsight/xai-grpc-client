use std::pin::Pin;
use tokio_stream::{Stream, StreamExt};
use crate::{
    error::{Result, GrokError},
    proto,
    request::ChatRequest,
    response::{ChatResponse, ChatChunk},
};
use super::config::GrokClient;

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

        let response = self.inner
            .get_completion_chunk(proto_request)
            .await?
            .into_inner();

        let stream = response.map(|result| {
            result
                .map_err(Into::into)
                .and_then(|chunk| Self::proto_chunk_to_chunk(chunk))
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

        self.inner
            .delete_stored_completion(proto_request)
            .await?;

        Ok(())
    }
}
