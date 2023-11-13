use network_tables::Value;

use crate::{
    nt::Key,
    trie::{NodeValue, Nodes},
};

pub trait NTValue {
    fn try_to_string(&self) -> Option<String>;
    fn try_to_string_array(&self) -> Option<Vec<String>>;
}

impl NTValue for Value {
    fn try_to_string(&self) -> Option<String> {
        if let Value::String(string) = self {
            string.clone().into_str()
        } else {
            None
        }
    }

    fn try_to_string_array(&self) -> Option<Vec<String>> {
        if let Value::Array(array) = self {
            array.iter().map(|value| value.try_to_string()).collect()
        } else {
            None
        }
    }
}

impl Nodes<Key, Value> {
    pub fn try_get_value(&self, key: &str) -> Option<&Value> {
        for node in &self.nodes {
            if node.key == key {
                return node.value.try_get_value();
            }
        }
        None
    }
}

impl NodeValue<Key, Value> {
    pub fn try_get_value(&self) -> Option<&Value> {
        if let NodeValue::Leaf(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

/*
fn expect_value(nodes: &Nodes<Key, Value>, key: &str) -> Result<Value, WidgetError> {
    for node in &nodes.nodes {
        if node.key == key {
            if let NodeValue::Leaf(value) = &node.value {
                return Ok(value.clone());
            } else {
                return Err(WidgetError::ExpectedValue(node.clone()));
            }
        }
    }
    Err(WidgetError::MissingField(key.to_string()))
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
*/
