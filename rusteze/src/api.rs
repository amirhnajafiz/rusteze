use axum::{ extract::{Json, Query, State}, routing::{ get, post }, Router };
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

use crate::memcache::MemCache;
use crate::requests::{ GetKeyRequest, GetKeyResponse, SetKeyRequest, SetKeyResponse };

pub struct APIServer {
    pub mem_cache: Arc<Mutex<MemCache>>,
}

impl APIServer {
    async fn http_handler_set_key(
        State(mem_cache): State<Arc<Mutex<MemCache>>>,
        Json(payload): Json<SetKeyRequest>,
    ) -> Json<SetKeyResponse> {
        println!("Received set key request: {:?}", payload);

        let mut cache = mem_cache.lock().await;
        cache.set(payload.key, payload.value, payload.ttl_seconds);
        Json(SetKeyResponse { success: true, time_to_live: payload.ttl_seconds })
    }

    // GET uses query params: /api/get?key=some_key
    async fn http_handler_get_key(
        State(mem_cache): State<Arc<Mutex<MemCache>>>,
        Query(params): Query<GetKeyRequest>,
    ) -> Json<GetKeyResponse> {
        println!("Received get key request for key: {}", params.key);

        let cache = mem_cache.lock().await;
        Json(GetKeyResponse {
            value: cache.get(&params.key),
        })
    }
}

impl APIServer {
    pub async fn start(&self, addr: SocketAddr) {
        let app = Router::new()
            .route("/api/set", post(Self::http_handler_set_key))
            .route("/api/get", get(Self::http_handler_get_key))
            .with_state(self.mem_cache.clone());

        let listener = TcpListener::bind(&addr).await.unwrap();

        axum::serve(listener, app).await.unwrap();
    }
}
