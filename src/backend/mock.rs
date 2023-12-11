use super::{Backend, Entry, Key, Path, Status, Update, Write};
use network_tables::rmpv::Integer;
use network_tables::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;
use std::time::Instant;

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

    fn write(&mut self, entries: Vec<Entry>) {
        for entry in entries {
            Tree::write(self, entry.path.into_vec(), entry.value);
        }
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
        if path.is_empty() {
            panic!("trying to write to folder {self:?}");
        } else {
            let key = path.remove(0);

            if let Some(existing) = self.get_mut(&key) {
                existing.write(path, value);
            } else if path.is_empty() {
                self.insert(key, Box::new(value));
            } else {
                let mut map = HashMap::new();
                Tree::write(&mut map, path, value);
                self.insert(key, Box::new(map));
            }
        }
    }
}

static START: Mutex<Option<Instant>> = Mutex::new(None);

impl Tree for Value {
    fn update(&mut self, path: Path) -> Update {
        let mut start = START.lock().unwrap();

        let is_old = if let Some(start_time) = start.as_ref() {
            start_time.elapsed().as_millis() > 500
        } else {
            *start = Some(Instant::now());
            false
        };

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

        if is_old {
            Update {
                to_update: vec![entry],
                to_create: Vec::new(),
            }
        } else {
            Update {
                to_update: Vec::new(),
                to_create: vec![entry],
            }
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
