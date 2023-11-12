use std::{
    fmt,
    net::{Ipv4Addr, SocketAddrV4},
    path::{Component, PathBuf},
    str::Utf8Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use network_tables::{
    v4::{Client, Subscription},
    Value,
};
use tokio::task::JoinHandle;

use crate::trie::{Keys, Trie};

pub type Key = String;

#[derive(Debug)]
pub enum KeyError {
    Encoding(Utf8Error),
    Empty,
}

pub fn from_nt_path(path: String) -> Result<Keys<Key, Vec<Key>>, KeyError> {
    let buf = PathBuf::from(path);
    let mut vec: Vec<String> = buf
        .components()
        .filter_map(|comp| {
            if let Component::Normal(str) = comp {
                Some(str.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    if vec.is_empty() {
        return Err(KeyError::Empty);
    }
    let first = vec.remove(0);
    Ok(Keys { first, rest: vec })
}

pub struct Backend {
    keys: Arc<Mutex<Trie<Key, Value>>>,
}

impl Backend {
    pub async fn new() -> Result<Self> {
        Ok(Backend {
            keys: Arc::new(Mutex::new(Trie::new())),
        })
    }
    /*
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
    */
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
                match from_nt_path(message.topic_name) {
                    Ok(path) => keys_inner.insert(path, message.data),
                    Err(error) => panic!("{:?}", error),
                }
                .unwrap();
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

    pub fn with_keys<T>(&self, keys: impl FnOnce(&Trie<String, Value>) -> T) -> T {
        keys(&self.keys.lock().unwrap())
    }
}

impl fmt::Debug for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(keys) = self.keys.lock() {
            write!(f, "{{")?;
            let mut is_first = true;
            let mut result = Ok(());

            keys.walk(&mut |keys, value| {
                if is_first {
                    is_first = false;
                } else {
                    result = write!(f, ", ").and(result);
                }
                let path = PathBuf::from(keys.join("/"));
                if let Some(str) = path.to_str() {
                    result = write!(f, "\"{}\": {}", str, value).and(result);
                } else {
                    result = write!(f, "\"{:?}\": {}", path, value).and(result);
                }
            });

            result?;

            write!(f, "}}")
        } else {
            write!(f, "Mutex Poisoning")
        }
    }
}
