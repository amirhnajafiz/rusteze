use serde::{ Deserialize, Serialize };

// Define request and response structures for the API endpoints.

// SetKeyRequest represents the payload for setting a key-value pair.
#[derive(Deserialize, Serialize, Debug)]
pub struct SetKeyRequest {
    pub key: String,
    pub value: String,
}

// SetKeyResponse represents the response for the set key operation.
#[derive(Deserialize, Serialize, Debug)]
pub struct SetKeyResponse {
    pub success: bool,
}

// GetKeyResponse represents the response for the get key operation.
#[derive(Deserialize, Serialize, Debug)]
pub struct GetKeyResponse {
    pub value: Option<String>,
}

// GetKeyRequest represents the query parameters for getting a key's value.
#[derive(Deserialize, Serialize, Debug)]
pub struct GetKeyRequest {
    pub key: String,
}
