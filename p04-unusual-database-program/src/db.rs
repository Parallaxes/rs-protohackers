use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct KVStore {
    db: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl KVStore {
    pub async fn new() -> Self {
        let db = Arc::new(Mutex::new(HashMap::<Vec<u8>, Vec<u8>>::new()));

        {
            let mut db_guard = db.lock().await;
            db_guard.insert(b"version".to_vec(), b"KVStore 2.0".to_vec());
        }

        Self { db }
    }

    pub async fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), Box<dyn Error>> {
        if key == b"version" {
            return Err("Key cannot be 'version'".into());
        }

        let mut db_guard = self.db.lock().await;
        db_guard.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    pub async fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let db_guard = self.db.lock().await;
        if let Some(value) = db_guard.get(key) {
            return Some(value.clone());
        }

        None
    }
}
