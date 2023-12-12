#![allow(dead_code)]

use core::fmt;
use std::{
    collections::HashSet,
    os::unix::prelude::OsStrExt,
    path::{Component, PathBuf},
};

use thiserror::Error;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};



use super::{
    backend::{Entry, Path, Status, StatusUpdate, Update},
    nt_worker::{run_worker},
    Backend, Key,
};
use anyhow::Result;

pub struct Nt {
    read_receiver: UnboundedReceiver<Entry>,
    write_sender: UnboundedSender<Entry>,
    status: Status,
    status_receiver: UnboundedReceiver<StatusUpdate>,
    paths: HashSet<Path>,
}

enum UpdateAction {
    Create(Entry),
    Update(Entry),
    End,
}

impl Backend for Nt {
    fn update(&mut self) -> Update {
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
                        to_update,
                        to_create,
                    }
                }
            }
        }
    }

    fn write(&mut self, entries: Vec<Entry>) {
        for entry in entries {
            self.write_update(entry);
        }
    }

    fn status(&self) -> Status {
        self.status
    }
}

impl Nt {
    pub fn new() -> Self {
        let (read_sender, read_receiver) = unbounded_channel();
        let (write_sender, write_receiver) = unbounded_channel();
        let (status_sender, status_receiver) = unbounded_channel();

        tokio::spawn(async move { run_worker(read_sender, write_receiver, status_sender).await });

        Self {
            read_receiver,
            write_sender,
            status: Status::default(),
            status_receiver,
            paths: HashSet::new(),
        }
    }

    fn nonblocking_update_poll(&mut self) -> UpdateAction {
        let Ok(Entry { path, value }) = self.read_receiver.try_recv() else {
            return UpdateAction::End;
        };
        if self.paths.contains(&path) {
            UpdateAction::Update(Entry { path, value })
        } else {
            self.paths.insert(path.clone());
            UpdateAction::Create(Entry { path, value })
        }
    }

    fn update_status(&mut self) {
        if let Ok(update) = self.status_receiver.try_recv() {
            self.status.update(update);
        }
    }

    fn write_update(&self, entry: Entry) {
        self.write_sender.send(entry).unwrap();
    }
}

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("Paths must have at least 1 component")]
    Empty,
}

pub fn from_nt_path(path: String) -> Result<Path, KeyError> {
    let buf = PathBuf::from(path);
    let mut vec: Vec<Key> = buf
        .components()
        .filter_map(|comp| {
            if let Component::Normal(str) = comp {
                Some(Key::from_utf8(str.as_bytes()).unwrap())
            } else {
                None
            }
        })
        .collect();
    if vec.is_empty() {
        return Err(KeyError::Empty);
    }
    let first = vec.remove(0);
    Ok(Path { first, rest: vec })
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.first)?;
        for comp in &self.rest {
            write!(f, "/{comp}")?;
        }
        Ok(())
    }
}
