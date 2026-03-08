use axum::{ extract::Query, extract::Json, routing::{ get, post }, Router };
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::requests::{ GetKeyRequest, GetKeyResponse, SetKeyRequest, SetKeyResponse };

pub struct APIServer {}

impl APIServer {
    async fn http_handler_set_key(Json(payload): Json<SetKeyRequest>) -> Json<SetKeyResponse> {
        println!("Received set key request: {:?}", payload);

        Json(SetKeyResponse { success: true })
    }

    // GET uses query params: /api/get?key=some_key
    async fn http_handler_get_key(Query(params): Query<GetKeyRequest>) -> Json<GetKeyResponse> {
        println!("Received get key request for key: {}", params.key);

        Json(GetKeyResponse {
            value: Some("example_value".to_string()),
        })
    }
}

impl APIServer {
    pub async fn start(&self, addr: SocketAddr) {
        let app = Router::new()
            .route("/api/set", post(Self::http_handler_set_key))
            .route("/api/get", get(Self::http_handler_get_key));

        let listener = TcpListener::bind(&addr).await.unwrap();

        axum::serve(listener, app).await.unwrap();
    }
}
