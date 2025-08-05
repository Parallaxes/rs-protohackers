use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

pub struct ChatRoom {
    users: Mutex<HashMap<String, UnboundedSender<String>>>,
}

impl ChatRoom {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            users: Mutex::new(HashMap::new()),
        })
    }

    pub async fn join(&self, name: String, tx: UnboundedSender<String>) {
        self.users.lock().await.insert(name, tx);
    }

    pub async fn leave(&self, name: &str) {
        self.users.lock().await.remove(name);
    }

    pub async fn broadcast(&self, msg: &str, except: Option<&str>) {
        let users = self.users.lock().await;
        for (user, tx) in users.iter() {
            if Some(user.as_str()) != except {
                let _ = tx.send(msg.to_string());
            }
        }
    }

    pub async fn user_list(&self, except: Option<&str>) -> Vec<String> {
        self.users
            .lock()
            .await
            .keys()
            .filter(|n| Some(n.as_str()) != except)
            .cloned()
            .collect()
    }
}
