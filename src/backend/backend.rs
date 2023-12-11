use std::{fmt, str::FromStr};

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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path {
    pub first: Key,
    pub rest: Vec<Key>,
}

impl Path {
    pub fn push(&mut self, arg: impl Into<Key>) {
        self.rest.push(arg.into());
    }

    pub fn into_vec(mut self) -> Vec<Key> {
        self.rest.insert(0, self.first);
        self.rest
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.first)?;

        for key in &self.rest {
            write!(f, "/{key}")?;
        }

        Ok(())
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

#[derive(Debug, Error)]
pub enum PathError {
    #[error("Path is empty")]
    Empty,
    #[error("Path {0} doesn't start with a /")]
    ExpectedSlash(String),
}

impl FromStr for Path {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut options = Vec::new();

        let mut current = String::new();

        let mut needs_slash = true;

        for char in s.chars() {
            if needs_slash {
                if char != '/' {
                    return Err(PathError::ExpectedSlash(s.to_string()));
                }
                needs_slash = false;
            } else if char == '/' {
                options.push(current);
                current = String::new();
            } else {
                current.push(char);
            }
        }
        options.push(current);

        options.try_into().map_err(|_| PathError::Empty)
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
