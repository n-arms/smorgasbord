use network_tables::Value;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Write {
    pub entries: Vec<Entry>,
}

impl Write {
    pub fn one(entry: Entry) -> Write {
        Write {
            entries: vec![entry],
        }
    }
}

pub type Key = String;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path {
    pub first: Key,
    pub rest: Vec<Key>,
}

impl Path {
    pub fn push(&mut self, arg: impl Into<Key>) {
        self.rest.push(arg.into());
    }

    pub fn to_vec(mut self) -> Vec<Key> {
        self.rest.insert(0, self.first);
        self.rest
    }
}

#[derive(Debug, Error)]
#[error("Vec is empty")]
pub struct Empty;

impl TryFrom<Vec<Key>> for Path {
    type Error = Empty;

    fn try_from(mut value: Vec<Key>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(Empty)
        } else {
            let first = value.remove(0);
            Ok(Self { first, rest: value })
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub path: Path,
    pub value: Value,
}

#[derive(Default)]
pub struct Update {
    pub to_update: Vec<Entry>,
    pub to_create: Vec<Entry>,
}

#[derive(Copy, Clone, Default)]
pub struct Status {
    pub is_connected: bool,
}

impl Status {
    pub fn update(&mut self, update: StatusUpdate) {
        match update {
            StatusUpdate::IsConnectedChange(is_connected) => self.is_connected = is_connected,
        }
    }
}

#[derive(Copy, Clone)]
pub enum StatusUpdate {
    IsConnectedChange(bool),
}

pub trait Backend {
    fn update(&mut self) -> Update;
    fn write(&mut self, write: Write);
    fn status(&self) -> Status;
}
