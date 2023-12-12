use std::{
    fmt,
    str::{from_utf8, FromStr, Utf8Error},
};

use network_tables::Value;
use smol_str::SmolStr;
use thiserror::Error;

use crate::widgets::tabs::Filter;

#[derive(Debug, Default)]
pub struct Write {
    entries: Vec<Entry>,
    filter: Option<Filter>,
}

impl Write {
    pub fn one(entry: Entry) -> Self {
        Self {
            entries: vec![entry],
            filter: None,
        }
    }

    pub fn filter(filter: Filter) -> Self {
        Self {
            entries: Vec::new(),
            filter: Some(filter),
        }
    }

    pub fn entries<'a>(&'a self) -> impl Iterator<Item = &'a Entry> {
        self.entries.iter()
    }

    pub fn try_filter(&self) -> Option<Filter> {
        self.filter.clone()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    inner: String,
}

impl From<&str> for Key {
    fn from(value: &str) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl Key {
    pub fn from_utf8(bytes: &[u8]) -> Result<Self, Utf8Error> {
        let inner = String::from(from_utf8(bytes)?);
        Ok(Self { inner })
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

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

        let mut buf = Vec::new();

        let mut needs_slash = true;

        for char in s.chars() {
            if needs_slash {
                if char != '/' {
                    return Err(PathError::ExpectedSlash(s.to_string()));
                }
                needs_slash = false;
            } else if char == '/' {
                options.push(Key::from_utf8(&buf).unwrap());
                buf.clear();
            } else {
                buf.push(char as u8);
            }
        }
        options.push(Key::from_utf8(&buf).unwrap());

        options.try_into().map_err(|_| PathError::Empty)
    }
}

#[derive(Clone, Debug)]
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
    fn write(&mut self, entries: Vec<Entry>);
    fn status(&self) -> Status;
}
