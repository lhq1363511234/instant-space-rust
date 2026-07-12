use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationNode {
    pub province: String,
    pub city: Option<String>,
    pub district: Option<String>,
    pub spot_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeoOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoMatch {
    pub country: String,
    pub province: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub spot_name: Option<String>,
    pub lat: f64,
    pub lng: f64,
}
