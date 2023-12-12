#![allow(dead_code)]

use super::{Backend, Entry, Key, Path, Status, Update};
use network_tables::rmpv::Integer;
use network_tables::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;
use std::time::Instant;

pub type T = Box<dyn Tree>;
pub type TMap = HashMap<Key, T>;

pub fn example_dashboard() -> TMap {
    let auto_type: T = Box::new(Value::String("String Chooser".into()));
    let auto_options: T = Box::new(Value::Array(vec![
        Value::String("Left".into()),
        Value::String("Right".into()),
    ]));
    let auto_selected: T = Box::new(Value::String("Left".into()));
    let auto_default: T = Box::new(Value::String("Left".into()));
    let auto: T = Box::new(map! {
        ".type".into() => auto_type,
        "options".into() => auto_options,
        "selected".into() => auto_selected,
        "default".into() => auto_default
    });
    let tabs_type: T = Box::new(Value::String("Tabs".into()));
    let drivetrain_option: T = Box::new(Value::Array(vec![
        Value::String("/Smartdashboard/left encoder".into()),
        Value::String("/Smartdashboard/right encoder".into()),
        Value::String("/Smartdashboard/gyro yaw".into()),
        Value::String("/Smartdashboard/kA".into()),
        Value::String("/Smartdashboard/kV".into()),
        Value::String("/Smartdashboard/kS".into()),
    ]));
    let arm_option: T = Box::new(Value::Array(vec![
        Value::String("/Smartdashboard/arm encoder".into()),
        Value::String("/Smartdashboard/arm current".into()),
        Value::String("/Smartdashboard/arm voltage".into()),
        Value::String("/Smartdashboard/arm kS".into()),
        Value::String("/Smartdashboard/arm kG".into()),
        Value::String("/Smartdashboard/arm kV".into()),
    ]));
    let auto_option: T = Box::new(Value::Array(vec![Value::String(
        "/Smartdashboard/auto".into(),
    )]));
    let tabs: T = Box::new(map! {
        ".type".into() => tabs_type,
        "drivetrain".into() => drivetrain_option,
        "arm".into() => arm_option,
        "auto".into() => auto_option
    });
    let counter: T = Box::new(Value::F32(0.0));
    let mut smartdashboard_map: TMap = map! {
        "counter".into() => counter,
        "auto".into() => auto,
        "tabs".into() => tabs
    };
    for name in [
        "left encoder",
        "right encoder",
        "gyro yaw",
        "through bore",
        "kA",
        "kV",
        "kS",
        "arm encoder",
        "arm current",
        "arm voltage",
        "arm kS",
        "arm kG",
        "arm kV",
    ] {
        let value: T = Box::new(Value::F32(0.0));
        smartdashboard_map.insert(name.into(), value);
    }

    let smartdashboard: T = Box::new(smartdashboard_map);
    map! {
        "Smartdashboard".into() => smartdashboard
    }
}

fn rand_string() -> Value {
    let mut string = String::new();
    for _ in 0..fastrand::usize(3..6) {
        string.push(fastrand::alphanumeric());
    }
    Value::String(string.into())
}

fn rand_key() -> Key {
    let mut buf = [0u8; 6];
    let size = fastrand::usize(3..6);
    for item in buf.iter_mut().take(size) {
        *item = fastrand::alphanumeric() as u8;
    }
    Key::from_utf8(&buf[..size]).unwrap()
}

fn chooser() -> T {
    let r#type: T = Box::new(Value::String("String Chooser".into()));
    let mut option_vec = Vec::new();
    for _ in 0..fastrand::usize(3..6) {
        option_vec.push(rand_string());
    }
    let selected: T = Box::new(option_vec[0].clone());
    let default: T = Box::new(option_vec[0].clone());
    let options: T = Box::new(Value::Array(option_vec));
    Box::new(map! {
        ".type".into() => r#type,
        "options".into() => options,
        "selected".into() => selected,
        "default".into() => default
    })
}

pub fn stressing_example(widgets: usize) -> TMap {
    let mut smartdashboard_map: TMap = TMap::new();

    for _ in 0..(widgets / 3) {
        smartdashboard_map.insert(rand_key(), chooser());
    }

    for _ in 0..(widgets - 2 * (widgets / 3)) {
        smartdashboard_map.insert(rand_key(), Box::new(Value::F32(0.0)));
    }

    let smartdashboard: T = Box::new(smartdashboard_map);
    map! {
        "Smartdashboard".into() => smartdashboard
    }
}

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
                inner_path.push(key.clone());
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
                if fastrand::bool() {
                    *value = Integer::from(value.as_i64().unwrap() + 1);
                } else if fastrand::bool() {
                    *value = Integer::from((value.as_f64().unwrap() * 0.9) as u64);
                }
            }
            Value::F32(value) => {
                if fastrand::bool() {
                    *value += 1.0;
                } else if fastrand::bool() {
                    *value *= 0.9;
                }
            }
            Value::F64(value) => {
                if fastrand::bool() {
                    *value += 1.0;
                } else if fastrand::bool() {
                    *value *= 0.9;
                }
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
