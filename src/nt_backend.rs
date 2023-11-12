use std::{
    collections::HashMap,
    fmt,
    net::{Ipv4Addr, SocketAddrV4},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use network_tables::{
    v4::{Client, Subscription},
    Value,
};
use tokio::task::JoinHandle;

pub type Key = String;

pub struct Backend {
    keys: Arc<Mutex<HashMap<Key, Value>>>,
}

impl Backend {
    pub async fn new() -> Result<Self> {
        Ok(Backend {
            keys: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    pub fn pairs(&self) -> impl IntoIterator<Item = (String, Value)> {
        let widgets: Vec<_> = self
            .keys
            .lock()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        widgets
    }
    pub async fn spawn_update_thread(&self) -> Result<JoinHandle<()>> {
        let keys = Arc::clone(&self.keys);
        let handle = tokio::spawn(async move {
            let mut client: Option<Client> = None;
            while client.is_none() {
                let maybe_client = network_tables::v4::Client::try_new_w_config(
                    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5810),
                    network_tables::v4::client_config::Config {
                        ..Default::default()
                    },
                )
                .await;
                if let Ok(c) = maybe_client {
                    client = Some(c);
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            let client = client.unwrap();
            let mut sub = Self::subscription(&client).await.unwrap();
            while let Some(message) = sub.next().await {
                let mut keys_inner = keys.lock().unwrap();
                keys_inner.insert(message.topic_name, message.data);
            }
        });
        Ok(handle)
    }

    async fn subscription(client: &Client) -> Result<Subscription> {
        client
            .subscribe_w_options(
                &["/SmartDashboard"],
                Some(network_tables::v4::SubscriptionOptions {
                    all: Some(true),
                    prefix: Some(true),
                    ..Default::default()
                }),
            )
            .await
            .map_err(Into::into)
    }

    pub fn with_keys<T>(&self, keys: impl FnOnce(&HashMap<String, Value>) -> T) -> T {
        keys(&self.keys.lock().unwrap())
    }
}

impl fmt::Debug for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(keys) = self.keys.lock() {
            write!(f, "{{")?;
            for (i, (name, value)) in keys.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "\"{}\": {}", name, value)?;
            }
            write!(f, "}}")
        } else {
            write!(f, "Mutex Poisoning")
        }
    }
}
