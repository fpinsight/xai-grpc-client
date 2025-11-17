//! API key information API
//!
//! This module provides access to your API key metadata and permissions.
//!
//! # Examples
//!
//! ```no_run
//! use xai_grpc_client::GrokClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = GrokClient::from_env().await?;
//!     let api_key_info = client.get_api_key_info().await?;
//!
//!     println!("API Key: {}", api_key_info.redacted_api_key);
//!     println!("Team ID: {}", api_key_info.team_id);
//!     println!("Blocked: {}", api_key_info.api_key_blocked);
//!
//!     Ok(())
//! }
//! ```

use crate::proto;

/// Information about an API key.
///
/// This struct contains metadata about your xAI API key, including its
/// permissions, team assignment, and status.
///
/// # Examples
///
/// ```no_run
/// # use xai_grpc_client::GrokClient;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut client = GrokClient::from_env().await?;
/// let info = client.get_api_key_info().await?;
///
/// // Check if key is active
/// if info.api_key_blocked || info.team_blocked || info.disabled {
///     println!("Warning: API key is not active!");
/// }
///
/// // Check permissions
/// println!("Permissions:");
/// for acl in &info.acls {
///     println!("  - {}", acl);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct ApiKeyInfo {
    /// Redacted API key for display purposes.
    ///
    /// The full API key is never returned after creation for security reasons.
    /// This field shows a partially masked version (e.g., "xai-abc...xyz").
    pub redacted_api_key: String,

    /// ID of the user who created this API key.
    pub user_id: String,

    /// Human-readable name for the API key.
    ///
    /// This name helps identify the purpose or application using the key.
    pub name: String,

    /// Unix timestamp (seconds) when the API key was created.
    pub created_at: i64,

    /// Unix timestamp (seconds) when the API key was last modified.
    pub modified_at: i64,

    /// ID of the last user who modified the API key.
    pub modified_by: String,

    /// ID of the team this API key belongs to.
    ///
    /// API keys are scoped to teams, and usage is tracked per team.
    pub team_id: String,

    /// Access Control Lists (ACLs) associated with this key.
    ///
    /// These permissions indicate which resources and operations
    /// the API key can access. Empty list means default permissions.
    pub acls: Vec<String>,

    /// The unique identifier for this API key (not the key itself).
    ///
    /// This ID can be used for API key management operations.
    pub api_key_id: String,

    /// Whether the API key is currently blocked from making requests.
    ///
    /// If `true`, all requests with this key will be rejected.
    pub api_key_blocked: bool,

    /// Whether the team is currently blocked from making requests.
    ///
    /// If `true`, all requests from this team (any API key) will be rejected.
    pub team_blocked: bool,

    /// Whether the API key is currently disabled.
    ///
    /// Disabled keys cannot make requests but can be re-enabled.
    pub disabled: bool,
}

impl ApiKeyInfo {
    /// Check if the API key is currently active and usable.
    ///
    /// Returns `true` if the key is not blocked, the team is not blocked,
    /// and the key is not disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use xai_grpc_client::ApiKeyInfo;
    /// # let info = ApiKeyInfo {
    /// #     redacted_api_key: "xai-***".to_string(),
    /// #     user_id: "user-123".to_string(),
    /// #     name: "Test Key".to_string(),
    /// #     created_at: 0,
    /// #     modified_at: 0,
    /// #     modified_by: "user-123".to_string(),
    /// #     team_id: "team-456".to_string(),
    /// #     acls: vec![],
    /// #     api_key_id: "key-789".to_string(),
    /// #     api_key_blocked: false,
    /// #     team_blocked: false,
    /// #     disabled: false,
    /// # };
    /// assert!(info.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        !self.api_key_blocked && !self.team_blocked && !self.disabled
    }

    /// Get a human-readable status string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use xai_grpc_client::ApiKeyInfo;
    /// # let info = ApiKeyInfo {
    /// #     redacted_api_key: "xai-***".to_string(),
    /// #     user_id: "user-123".to_string(),
    /// #     name: "Test Key".to_string(),
    /// #     created_at: 0,
    /// #     modified_at: 0,
    /// #     modified_by: "user-123".to_string(),
    /// #     team_id: "team-456".to_string(),
    /// #     acls: vec![],
    /// #     api_key_id: "key-789".to_string(),
    /// #     api_key_blocked: false,
    /// #     team_blocked: false,
    /// #     disabled: false,
    /// # };
    /// assert_eq!(info.status_string(), "Active");
    /// ```
    pub fn status_string(&self) -> &'static str {
        if self.api_key_blocked {
            "Blocked (Key)"
        } else if self.team_blocked {
            "Blocked (Team)"
        } else if self.disabled {
            "Disabled"
        } else {
            "Active"
        }
    }
}

impl From<proto::ApiKey> for ApiKeyInfo {
    fn from(proto: proto::ApiKey) -> Self {
        Self {
            redacted_api_key: proto.redacted_api_key,
            user_id: proto.user_id,
            name: proto.name,
            created_at: proto.create_time.map(|t| t.seconds).unwrap_or(0),
            modified_at: proto.modify_time.map(|t| t.seconds).unwrap_or(0),
            modified_by: proto.modified_by,
            team_id: proto.team_id,
            acls: proto.acls,
            api_key_id: proto.api_key_id,
            api_key_blocked: proto.api_key_blocked,
            team_blocked: proto.team_blocked,
            disabled: proto.disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_api_key_info() -> ApiKeyInfo {
        ApiKeyInfo {
            redacted_api_key: "xai-abc***xyz".to_string(),
            user_id: "user-123".to_string(),
            name: "Test API Key".to_string(),
            created_at: 1704067200,  // 2024-01-01 00:00:00 UTC
            modified_at: 1704153600, // 2024-01-02 00:00:00 UTC
            modified_by: "user-123".to_string(),
            team_id: "team-456".to_string(),
            acls: vec!["chat:read".to_string(), "chat:write".to_string()],
            api_key_id: "key-789".to_string(),
            api_key_blocked: false,
            team_blocked: false,
            disabled: false,
        }
    }

    #[test]
    fn test_is_active_when_all_flags_false() {
        let info = create_test_api_key_info();
        assert!(info.is_active());
    }

    #[test]
    fn test_is_not_active_when_api_key_blocked() {
        let mut info = create_test_api_key_info();
        info.api_key_blocked = true;
        assert!(!info.is_active());
    }

    #[test]
    fn test_is_not_active_when_team_blocked() {
        let mut info = create_test_api_key_info();
        info.team_blocked = true;
        assert!(!info.is_active());
    }

    #[test]
    fn test_is_not_active_when_disabled() {
        let mut info = create_test_api_key_info();
        info.disabled = true;
        assert!(!info.is_active());
    }

    #[test]
    fn test_status_string_active() {
        let info = create_test_api_key_info();
        assert_eq!(info.status_string(), "Active");
    }

    #[test]
    fn test_status_string_blocked_key() {
        let mut info = create_test_api_key_info();
        info.api_key_blocked = true;
        assert_eq!(info.status_string(), "Blocked (Key)");
    }

    #[test]
    fn test_status_string_blocked_team() {
        let mut info = create_test_api_key_info();
        info.team_blocked = true;
        assert_eq!(info.status_string(), "Blocked (Team)");
    }

    #[test]
    fn test_status_string_disabled() {
        let mut info = create_test_api_key_info();
        info.disabled = true;
        assert_eq!(info.status_string(), "Disabled");
    }

    #[test]
    fn test_api_key_info_clone() {
        let info = create_test_api_key_info();
        let cloned = info.clone();

        assert_eq!(info.redacted_api_key, cloned.redacted_api_key);
        assert_eq!(info.user_id, cloned.user_id);
        assert_eq!(info.team_id, cloned.team_id);
    }

    #[test]
    fn test_api_key_info_debug() {
        let info = create_test_api_key_info();
        let debug_str = format!("{info:?}");
        assert!(debug_str.contains("Test API Key"));
        assert!(debug_str.contains("team-456"));
    }

    #[test]
    fn test_from_proto() {
        let proto_key = proto::ApiKey {
            redacted_api_key: "xai-test***key".to_string(),
            user_id: "user-789".to_string(),
            name: "Proto Test Key".to_string(),
            create_time: Some(prost_types::Timestamp {
                seconds: 1700000000,
                nanos: 0,
            }),
            modify_time: Some(prost_types::Timestamp {
                seconds: 1700100000,
                nanos: 0,
            }),
            modified_by: "user-789".to_string(),
            team_id: "team-abc".to_string(),
            acls: vec!["read".to_string(), "write".to_string()],
            api_key_id: "key-xyz".to_string(),
            api_key_blocked: false,
            team_blocked: true,
            disabled: false,
        };

        let info: ApiKeyInfo = proto_key.into();

        assert_eq!(info.redacted_api_key, "xai-test***key");
        assert_eq!(info.user_id, "user-789");
        assert_eq!(info.name, "Proto Test Key");
        assert_eq!(info.created_at, 1700000000);
        assert_eq!(info.modified_at, 1700100000);
        assert_eq!(info.modified_by, "user-789");
        assert_eq!(info.team_id, "team-abc");
        assert_eq!(info.acls.len(), 2);
        assert_eq!(info.api_key_id, "key-xyz");
        assert!(!info.api_key_blocked);
        assert!(info.team_blocked);
        assert!(!info.disabled);
        assert!(!info.is_active());
    }

    #[test]
    fn test_from_proto_with_none_timestamps() {
        let proto_key = proto::ApiKey {
            redacted_api_key: "xai-***".to_string(),
            user_id: "user".to_string(),
            name: "Key".to_string(),
            create_time: None,
            modify_time: None,
            modified_by: "user".to_string(),
            team_id: "team".to_string(),
            acls: vec![],
            api_key_id: "key".to_string(),
            api_key_blocked: false,
            team_blocked: false,
            disabled: false,
        };

        let info: ApiKeyInfo = proto_key.into();
        assert_eq!(info.created_at, 0);
        assert_eq!(info.modified_at, 0);
    }
}
