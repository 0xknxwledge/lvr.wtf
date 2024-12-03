use std::sync::Arc;
use object_store::ObjectStore;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn ObjectStore>,
}

impl AppState {
    pub fn new(store: Arc<dyn ObjectStore>) -> Self {
        Self { store }
    }
}