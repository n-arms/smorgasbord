use super::{Backend, Entry, Key, Path, Status, Update, Write};
use network_tables::rmpv::Integer;
use network_tables::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::{event, Level};

pub type T = Box<dyn Tree>;
pub type TMap = HashMap<Key, T>;

impl Backend for HashMap<Key, Box<dyn Tree>> {
    fn update(&mut self) -> Update {
        let mut to_update = Vec::new();
        let mut to_create = Vec::new();
        for (key, value) in self.iter_mut() {
            let update = value.update(Path {
                first: key.clone(),
                rest: Vec::new(),
            });
            to_update.extend(update.to_update);
            to_create.extend(update.to_create);
        }
        Update {
            to_update,
            to_create,
        }
    }

    fn write(&mut self, write: Write) {
        event!(Level::INFO, "writing {:?}", write);
        for entry in write.entries {
            Tree::write(self, entry.path.to_vec(), entry.value);
        }
        event!(Level::INFO, "after writing is {:?}", self);
    }

    fn status(&self) -> Status {
        Status { is_connected: true }
    }
}

pub trait Tree: Debug {
    fn update(&mut self, path: Path) -> Update;
    fn write(&mut self, path: Vec<Key>, value: Value);
}

impl Tree for HashMap<Key, Box<dyn Tree>> {
    fn update(&mut self, path: Path) -> Update {
        self.iter_mut()
            .map(|(key, tree)| {
                let mut inner_path = path.clone();
                inner_path.push(key);
                tree.update(inner_path)
            })
            .fold(Update::default(), |mut acc, x| {
                acc.to_update.extend(x.to_update);
                acc.to_create.extend(x.to_create);
                acc
            })
    }

    fn write(&mut self, mut path: Vec<Key>, value: Value) {
        event!(
            Level::INFO,
            "writing {} to {:?} with path {:?}",
            value,
            self,
            path
        );
        if path.is_empty() {
            panic!("trying to write to folder {:?}", self);
        } else {
            let key = path.remove(0);

            if let Some(existing) = self.get_mut(&key) {
                existing.write(path, value);
            } else {
                if path.is_empty() {
                    self.insert(key, Box::new(value));
                } else {
                    let mut map = HashMap::new();
                    Tree::write(&mut map, path, value);
                    self.insert(key, Box::new(map));
                }
            }
        }
    }
}

impl Tree for Value {
    fn update(&mut self, path: Path) -> Update {
        match self {
            Value::Integer(value) => {
                *value = Integer::from(value.as_i64().unwrap() + 1);
            }
            Value::F32(value) => {
                *value += 1.0;
            }
            Value::F64(value) => {
                *value += 1.0;
            }
            _ => {}
        }
        let entry = Entry {
            path,
            value: self.clone(),
        };
        Update {
            to_update: Vec::new(),
            to_create: vec![entry],
        }
    }

    fn write(&mut self, path: Vec<Key>, value: Value) {
        if path.is_empty() {
            *self = value;
        } else {
            panic!("trying to write {path:?} to plain value");
        }
    }
}
