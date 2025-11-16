use secrecy::{ExposeSecret, SecretString};
use tonic::{Request, Status};

#[derive(Clone)]
pub struct AuthInterceptor {
    api_key: SecretString,
}

impl AuthInterceptor {
    pub fn new(api_key: SecretString) -> Self {
        Self { api_key }
    }
}

impl tonic::service::Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = format!("Bearer {}", self.api_key.expose_secret());
        let metadata_value = token
            .parse()
            .map_err(|e| Status::internal(format!("Invalid auth token: {}", e)))?;

        request
            .metadata_mut()
            .insert("authorization", metadata_value);

        Ok(request)
    }
}
