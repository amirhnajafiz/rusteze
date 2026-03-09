use axum::{ extract::{Json, Query, State}, routing::{ get, post }, Router };
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tracing::{ info, error};

use crate::memcache::MemCache;
use crate::requests::{ GetKeyRequest, GetKeyResponse, SetKeyRequest, SetKeyResponse };

// APIServer struct to hold the shared state (mem_cache) and define the API handlers.
pub struct APIServer {
    pub mem_cache: Arc<Mutex<MemCache>>,
}

impl APIServer {
    // POST /api/set expects a JSON body with key, value, and optional ttl_seconds.
    async fn http_handler_set_key(
        State(mem_cache): State<Arc<Mutex<MemCache>>>,
        Json(payload): Json<SetKeyRequest>,
    ) -> Json<SetKeyResponse> {
        // validate the input (key and value should not be empty)
        if payload.key.is_empty() || payload.value.is_empty() {
            error!("invalid input: key and value must not be empty");
            return Json(SetKeyResponse { success: false, time_to_live: None });
        }

        info!("key `{}` set successfully with TTL: {:?}", payload.key, payload.ttl_seconds);

        // set the key-value pair in the cache with the specified TTL
        let mut cache = mem_cache.lock().await;
        cache.set(payload.key, payload.value, payload.ttl_seconds);

        // return a success response with the TTL information
        Json(SetKeyResponse { success: true, time_to_live: payload.ttl_seconds })
    }

    // GET /api/get expects a query parameter with the key to retrieve.
    async fn http_handler_get_key(
        State(mem_cache): State<Arc<Mutex<MemCache>>>,
        Query(params): Query<GetKeyRequest>,
    ) -> Json<GetKeyResponse> {
        let mut cache = mem_cache.lock().await;
        Json(GetKeyResponse {
            value: cache.get(&params.key),
        })
    }
}

impl APIServer {
    // start the API server on the specified address.
    pub async fn start(&self, addr: SocketAddr) {
        let app = Router::new()
            .route("/api/set", post(Self::http_handler_set_key))
            .route("/api/get", get(Self::http_handler_get_key))
            .route("/health", get(|| async { "OK" }))
            .route("/ready", get(|| async { "OK" }))
            .with_state(self.mem_cache.clone());

        let listener = TcpListener::bind(&addr).await.unwrap();

        axum::serve(listener, app).await.unwrap();
    }
}
