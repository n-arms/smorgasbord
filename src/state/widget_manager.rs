use network_tables::{rmpv::Utf8String, Value};

use crate::widgets::{Builder, Widget};
use crate::{
    nt::Key,
    trie::{Node, NodeValue, Nodes, Trie},
};

#[derive(Clone, Debug)]
pub enum WidgetError {
    MissingField(String),
    ExpectedValue(Node<Key, Value>),
    IncorrectField(Value, String),
    EncodingError(Utf8String),
}

#[derive(Default)]
pub struct WidgetManager {
    builders: Vec<Box<dyn Builder>>,
}

impl WidgetManager {
    pub fn new(builders: Vec<Box<dyn Builder>>) -> Self {
        Self { builders }
    }

    pub fn with(mut self, builder: impl Builder + 'static) -> Self {
        self.builders.push(Box::new(builder));
        self
    }

    pub fn add(&mut self, builder: impl Builder + 'static) {
        self.builders.push(Box::new(builder))
    }

    pub fn widgets(&self, data: &Trie<Key, Value>) -> Vec<Widget> {
        let mut output = Vec::new();
        self.visit_nodes(&data.root, Vec::new(), &mut output);
        output
    }

    pub fn visit_nodes(
        &self,
        data: &Nodes<Key, Value>,
        prefix: Vec<String>,
        output: &mut Vec<Widget>,
    ) {
        for node in &data.nodes {
            self.visit_node(node, prefix.clone(), output);
        }
    }

    pub fn visit_node(
        &self,
        data: &Node<Key, Value>,
        mut prefix: Vec<String>,
        output: &mut Vec<Widget>,
    ) {
        prefix.push(data.key.clone());

        for builder in &self.builders {
            let Some(kind) = builder.create_kind(&data.value) else {
                continue;
            };
            let widget = Widget::new(prefix.join("/"), kind);
            output.push(widget);
            return;
        }

        if let NodeValue::Branch(nodes) = &data.value {
            self.visit_nodes(nodes, prefix, output);
        }
    }
}
/*
pub fn make_widgets(data: &Trie<Key, Value>) -> impl IntoIterator<Item = Widget> {
    let mut output = Vec::new();
    nodes_into_widgets(&data.root, Vec::new(), &mut output);
    output
}

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
/*
fn string_chooser(nodes: &Nodes<Key, Value>) -> Result<Option<WidgetKind>, WidgetError> {
    let Ok(type_value) = expect_value(nodes, ".type") else {
        return Ok(None);
    };
    let Ok(r#type) = expect_string(type_value) else {
        return Ok(None);
    };
    if r#type != "String Chooser" {
        return Ok(None);
    }
    let options = expect_value(nodes, "options")?;
    let default = expect_value(nodes, "default")?;
    let active = expect_value(nodes, "active")?;

    Ok(Some(WidgetKind::Chooser {
        options: expect_string_array(options)?,
        default: expect_string(default)?,
        active: expect_string(active)?,
    }))
}
*/

fn nodes_into_widgets(nodes: &Nodes<Key, Value>, prefix: Vec<String>, output: &mut Vec<Widget>) {
    /*
    if let Some(widget_kind) = string_chooser(nodes).unwrap() {
        let title_path = PathBuf::from(prefix.join("/"));
        output.push(Widget {
            title: title_path.to_str().unwrap().to_string(),
            value: widget_kind,
        });
        return;
    }
        */
    for node in &nodes.nodes {
        node_into_widgets(node, prefix.clone(), output);
    }
}

fn node_into_widgets(node: &Node<Key, Value>, mut prefix: Vec<String>, output: &mut Vec<Widget>) {
    prefix.push(node.key.clone());

    match &node.value {
        NodeValue::Leaf(value) => {
            let title = PathBuf::from(prefix.join("/"))
                .to_str()
                .unwrap()
                .to_string();
            /*
            let widget = WidgetKind::Simple {
                value: Some(value.clone()),
            };
            output.push(Widget {
                title,
                value: widget,
            });
            */
        }
        NodeValue::Branch(branches) => {
            nodes_into_widgets(branches, prefix, output);
        }
    }
}
*/
