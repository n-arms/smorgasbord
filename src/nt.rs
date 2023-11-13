use core::fmt;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::{Component, PathBuf},
    time::Duration,
};

use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};

use network_tables::{
    v4::{Client, Subscription},
    Value,
};

use crate::trie::{Keys, Trie};
use anyhow::Result;

pub type Path = Keys<Key, Vec<Key>>;

const UPDATE_CHANNEL_SIZE: usize = 128;

#[derive(Copy, Clone, Default)]
pub struct Status {
    pub is_connected: bool,
}

pub struct Backend {
    pub trie: Trie<Key, Value>,
    updates: Receiver<(Path, Value)>,
    network_thread: JoinHandle<()>,
    pub status: Status,
    status_updates: Receiver<Status>,
}

enum UpdateAction {
    Create(Entry),
    Update(Entry),
    End,
}

pub struct Entry {
    pub path: Path,
    pub value: Value,
}

pub struct Update {
    pub to_update: Vec<Entry>,
    pub to_create: Vec<Entry>,
}

impl Backend {
    pub async fn new() -> Self {
        let status = Status::default();
        let (updates, network_thread, status_updates) = update_thread();

        Self {
            trie: Trie::new(),
            updates,
            network_thread,
            status,
            status_updates,
        }
    }

    pub fn update(&mut self) -> Update {
        self.update_status();
        let mut to_create = Vec::new();
        let mut to_update = Vec::new();
        loop {
            let action = self.nonblocking_update_poll();
            match action {
                UpdateAction::Create(entry) => to_create.push(entry),
                UpdateAction::Update(entry) => to_update.push(entry),
                UpdateAction::End => {
                    return Update {
                        to_create,
                        to_update,
                    }
                }
            }
        }
    }

    fn nonblocking_update_poll(&mut self) -> UpdateAction {
        let Ok((path, value)) = self.updates.try_recv() else {
            return UpdateAction::End;
        };
        let result = self.trie.insert(path.clone(), value.clone()).unwrap(); // TODO
        match result {
            Some(_) => UpdateAction::Update(Entry { path, value }),
            None => UpdateAction::Create(Entry { path, value }),
        }
    }

    fn update_status(&mut self) {
        if let Ok(new_status) = self.status_updates.try_recv() {
            self.status = new_status;
        }
    }
}

fn update_thread() -> (Receiver<(Path, Value)>, JoinHandle<()>, Receiver<Status>) {
    let (sender, receiver) = channel(UPDATE_CHANNEL_SIZE);
    let (status_send, status_recv) = channel(UPDATE_CHANNEL_SIZE);
    let handle = tokio::spawn(async move {
        let mut status = Status::default();
        loop {
            status.is_connected = false;
            status_send.send(status).await.unwrap();
            let client =
                connect_to_client(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5810)).await;
            status.is_connected = true;
            status_send.send(status).await.unwrap();
            let sub = subscribe(&client).await.unwrap(); // TODO
            forward_messages(sub, &sender).await;
        }
    });
    (receiver, handle, status_recv)
}

async fn subscribe(client: &Client) -> Result<Subscription> {
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

async fn forward_messages(mut sub: Subscription, sender: &Sender<(Path, Value)>) {
    while let Some(message) = sub.next().await {
        sender
            .send((from_nt_path(message.topic_name).unwrap(), message.data))
            .await
            .unwrap(); // TODO
    }
}

async fn connect_to_client(new: SocketAddrV4) -> Client {
    loop {
        let maybe_client = network_tables::v4::Client::try_new_w_config(
            SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5810),
            network_tables::v4::client_config::Config {
                ..Default::default()
            },
        )
        .await;
        if let Ok(c) = maybe_client {
            return c;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

pub type Key = String;

#[derive(Debug)]
pub enum KeyError {
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

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.first.fmt(f)?;
        for comp in &self.rest {
            write!(f, "/{}", comp)?;
        }
        Ok(())
    }
}
