use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub user_id: String,
    pub display_name: String,
    pub name_set: bool,
    /// Optional Supabase Auth JWT for authenticated connections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

impl Identity {
    pub fn generate(hostname: &str) -> Self {
        Self {
            user_id: Uuid::new_v4().to_string(),
            display_name: hostname.to_string(),
            name_set: false,
            access_token: None,
        }
    }

    /// Create an identity from a Supabase Auth session.
    pub fn from_supabase_auth(user_id: String, display_name: String, access_token: String) -> Self {
        Self {
            user_id,
            display_name,
            name_set: true,
            access_token: Some(access_token),
        }
    }
}
