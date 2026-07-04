use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationNode {
    pub province: String,
    pub city: Option<String>,
    pub district: Option<String>,
    pub spot_name: Option<String>,
}
