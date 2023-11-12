use std::{
    collections::HashMap,
    path::{Component, PathBuf},
};

use network_tables::{rmpv::Utf8String, Value};

use crate::{
    nt_backend::Key,
    widget::{Widget, WidgetKind},
};

#[derive(Clone, Debug)]
enum TableData {
    Value(Value),
    Nested(HashMap<Key, TableData>),
}

impl TableData {
    fn new(data: &HashMap<Key, Value>) -> Self {
        let mut result = Self::Nested(HashMap::new());

        for (key, value) in data {
            let path = PathBuf::from(key);

            let components = path.components().filter_map(|comp| {
                if let Component::Normal(text) = comp {
                    text.to_str().map(str::to_string)
                } else {
                    None
                }
            });

            result.add_value(components, value.clone());
        }

        result
    }

    fn add_value(&mut self, mut keys: impl Iterator<Item = Key>, value: Value) {
        match self {
            TableData::Value(old_value) => {
                if let Some(key) = keys.next() {
                    let mut map = HashMap::new();
                    let mut inner = TableData::Nested(HashMap::new());
                    inner.add_value(keys, value);
                    map.insert(key, inner);
                    *self = TableData::Nested(map);
                } else {
                    *old_value = value;
                }
            }
            TableData::Nested(map) => {
                if let Some(key) = keys.next() {
                    if let Some(inner) = map.get_mut(&key) {
                        inner.add_value(keys, value);
                    } else {
                        let mut inner = TableData::Nested(HashMap::new());
                        inner.add_value(keys, value);
                        map.insert(key, inner);
                    }
                } else {
                    *self = TableData::Value(value);
                }
            }
        }
    }

    fn widgets(self, path: PathBuf, widgets: &mut Vec<Widget>) {
        match self {
            TableData::Value(value) => widgets.push(Widget {
                title: path.into_os_string().into_string().unwrap(),
                value: WidgetKind::Simple { value: Some(value) },
            }),
            TableData::Nested(map) => {
                if let Ok(r#type) = expect_type(&map) {
                    if &r#type == "String Chooser" {
                        let kind = string_chooser(map).unwrap();
                        widgets.push(Widget {
                            title: path.into_os_string().into_string().unwrap(),
                            value: kind,
                        });
                        return;
                    }
                }

                for (key, data) in map {
                    let mut inner_path = path.clone();
                    inner_path.push(key);
                    data.widgets(inner_path, widgets);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum WidgetError {
    MissingField(String),
    ExpectedValue(HashMap<String, TableData>),
    IncorrectField(Value, String),
    EncodingError(Utf8String),
}

fn expect_type(map: &HashMap<String, TableData>) -> Result<String, WidgetError> {
    expect_string(expect_value(map, ".type")?)
}

fn expect_value(map: &HashMap<String, TableData>, key: &str) -> Result<Value, WidgetError> {
    let value = map
        .get(key)
        .ok_or_else(|| WidgetError::MissingField(key.to_owned()))?;

    match value {
        TableData::Value(value) => Ok(value.clone()),
        TableData::Nested(map) => Err(WidgetError::ExpectedValue(map.clone())),
    }
}

fn expect_string(value: Value) -> Result<String, WidgetError> {
    match value {
        Value::String(string) => {
            let error = WidgetError::EncodingError(string.clone());
            string.into_str().ok_or(error)
        }
        _ => Err(WidgetError::IncorrectField(value, "string".to_string())),
    }
}

fn expect_string_array(value: Value) -> Result<Vec<String>, WidgetError> {
    match value {
        Value::Array(array) => array.into_iter().map(expect_string).collect(),
        _ => Err(WidgetError::IncorrectField(value, "string[]".to_string())),
    }
}

fn string_chooser(map: HashMap<String, TableData>) -> Result<WidgetKind, WidgetError> {
    let active = expect_value(&map, "active")?;
    let options = expect_value(&map, "options")?;
    let default = expect_value(&map, "default")?;

    Ok(WidgetKind::Chooser {
        options: expect_string_array(options)?,
        default: expect_string(default)?,
        active: expect_string(active)?,
    })
}

pub fn make_widgets(data: &HashMap<Key, Value>) -> impl IntoIterator<Item = Widget> {
    let table = TableData::new(data);
    let mut widgets = Vec::new();
    table.widgets(PathBuf::new(), &mut widgets);
    widgets
}
