//! Image generation API
//!
//! Generate images from text prompts using xAI's image generation models.

use crate::proto;

/// Request for image generation
#[derive(Debug, Clone)]
pub struct ImageGenerationRequest {
    /// Text prompt describing the image
    pub prompt: String,
    /// Optional source image URL
    pub image_url: Option<String>,
    /// Model name
    pub model: String,
    /// Number of images to generate (1-10)
    pub n: Option<i32>,
    /// User identifier
    pub user: Option<String>,
    /// Output format (base64 or URL)
    pub format: ImageFormat,
}

/// Image output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Base64-encoded string
    Base64,
    /// URL to download image
    Url,
}

impl ImageGenerationRequest {
    /// Create a new image generation request
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            image_url: None,
            model: model.into(),
            n: None,
            user: None,
            format: ImageFormat::Url,
        }
    }

    /// Set the number of images to generate
    pub fn with_n(mut self, n: i32) -> Self {
        self.n = Some(n);
        self
    }

    /// Set source image for image-to-image generation
    pub fn with_source_image(mut self, url: impl Into<String>) -> Self {
        self.image_url = Some(url.into());
        self
    }

    /// Set output format
    pub fn with_format(mut self, format: ImageFormat) -> Self {
        self.format = format;
        self
    }

    /// Set user identifier
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

/// Response from image generation
#[derive(Debug, Clone)]
pub struct ImageGenerationResponse {
    /// Generated images
    pub images: Vec<GeneratedImage>,
    /// Model used
    pub model: String,
}

/// A generated image
#[derive(Debug, Clone)]
pub struct GeneratedImage {
    /// Base64-encoded image data (if format is Base64)
    pub base64: Option<String>,
    /// URL to image (if format is URL)
    pub url: Option<String>,
    /// Upsampled prompt used
    pub upsampled_prompt: String,
    /// Whether image respects moderation rules
    pub respects_moderation: bool,
}

impl From<proto::ImageResponse> for ImageGenerationResponse {
    fn from(proto: proto::ImageResponse) -> Self {
        Self {
            images: proto.images.into_iter().map(Into::into).collect(),
            model: proto.model,
        }
    }
}

impl From<proto::GeneratedImage> for GeneratedImage {
    fn from(proto: proto::GeneratedImage) -> Self {
        let (base64, url) = match proto.image {
            Some(proto::generated_image::Image::Base64(b64)) => (Some(b64), None),
            Some(proto::generated_image::Image::Url(u)) => (None, Some(u)),
            None => (None, None),
        };

        Self {
            base64,
            url,
            upsampled_prompt: proto.up_sampled_prompt,
            respects_moderation: proto.respect_moderation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_request_builder() {
        let request = ImageGenerationRequest::new("image-gen-1", "A sunset over mountains")
            .with_n(3)
            .with_source_image("https://example.com/source.jpg")
            .with_format(ImageFormat::Base64)
            .with_user("user-123");

        assert_eq!(request.model, "image-gen-1");
        assert_eq!(request.prompt, "A sunset over mountains");
        assert_eq!(request.n, Some(3));
        assert_eq!(request.image_url, Some("https://example.com/source.jpg".to_string()));
        assert_eq!(request.format, ImageFormat::Base64);
        assert_eq!(request.user, Some("user-123".to_string()));
    }

    #[test]
    fn test_image_request_minimal() {
        let request = ImageGenerationRequest::new("image-gen-1", "A cat");

        assert_eq!(request.model, "image-gen-1");
        assert_eq!(request.prompt, "A cat");
        assert_eq!(request.n, None);
        assert_eq!(request.image_url, None);
        assert_eq!(request.format, ImageFormat::Url);
        assert_eq!(request.user, None);
    }

    #[test]
    fn test_image_format() {
        assert_eq!(ImageFormat::Base64, ImageFormat::Base64);
        assert_eq!(ImageFormat::Url, ImageFormat::Url);
        assert_ne!(ImageFormat::Base64, ImageFormat::Url);
    }

    #[test]
    fn test_generated_image_from_proto_base64() {
        let proto_image = proto::GeneratedImage {
            image: Some(proto::generated_image::Image::Base64("base64data".to_string())),
            up_sampled_prompt: "Enhanced prompt".to_string(),
            respect_moderation: true,
        };

        let image: GeneratedImage = proto_image.into();
        assert_eq!(image.base64, Some("base64data".to_string()));
        assert_eq!(image.url, None);
        assert_eq!(image.upsampled_prompt, "Enhanced prompt");
        assert!(image.respects_moderation);
    }

    #[test]
    fn test_generated_image_from_proto_url() {
        let proto_image = proto::GeneratedImage {
            image: Some(proto::generated_image::Image::Url("https://example.com/image.jpg".to_string())),
            up_sampled_prompt: "Enhanced prompt".to_string(),
            respect_moderation: false,
        };

        let image: GeneratedImage = proto_image.into();
        assert_eq!(image.base64, None);
        assert_eq!(image.url, Some("https://example.com/image.jpg".to_string()));
        assert_eq!(image.upsampled_prompt, "Enhanced prompt");
        assert!(!image.respects_moderation);
    }

    #[test]
    fn test_generated_image_from_proto_none() {
        let proto_image = proto::GeneratedImage {
            image: None,
            up_sampled_prompt: "Prompt".to_string(),
            respect_moderation: true,
        };

        let image: GeneratedImage = proto_image.into();
        assert_eq!(image.base64, None);
        assert_eq!(image.url, None);
    }

    #[test]
    fn test_image_response_from_proto() {
        let proto_response = proto::ImageResponse {
            images: vec![
                proto::GeneratedImage {
                    image: Some(proto::generated_image::Image::Url("url1".to_string())),
                    up_sampled_prompt: "Prompt 1".to_string(),
                    respect_moderation: true,
                },
                proto::GeneratedImage {
                    image: Some(proto::generated_image::Image::Base64("data2".to_string())),
                    up_sampled_prompt: "Prompt 2".to_string(),
                    respect_moderation: true,
                },
            ],
            model: "image-gen-1".to_string(),
        };

        let response: ImageGenerationResponse = proto_response.into();
        assert_eq!(response.model, "image-gen-1");
        assert_eq!(response.images.len(), 2);
        assert_eq!(response.images[0].url, Some("url1".to_string()));
        assert_eq!(response.images[1].base64, Some("data2".to_string()));
    }

    #[test]
    fn test_image_request_clone() {
        let request = ImageGenerationRequest::new("model", "prompt")
            .with_n(2)
            .with_format(ImageFormat::Base64);

        let cloned = request.clone();
        assert_eq!(cloned.model, request.model);
        assert_eq!(cloned.prompt, request.prompt);
        assert_eq!(cloned.n, request.n);
        assert_eq!(cloned.format, request.format);
    }
}
