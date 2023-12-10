use crate::{
    backend::{Entry, Key, Path},
    widgets::Widget,
    widgets::{self, BuildResult, Builder},
};

use anyhow::Result;
use thiserror::Error;
use tracing::{event, Level};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Expected a value")]
    ExpectedValue,
    #[error("Expected a branch")]
    ExpectedBranch,
}

#[derive(Copy, Clone, Debug)]
pub struct BuilderIndex {
    index: usize,
}

pub struct Node {
    pub key: Key,
    pub widgets: Vec<Widget>,
    pub partial_widgets: Vec<BuilderIndex>,
    pub value: Value,
}

impl Node {
    pub fn new<'a>(
        mut path: Vec<Key>,
        first: &Key,
        rest: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Self {
        path.push(first.clone());
        let value_node = Value::new(path.clone(), rest, value, builders);
        let (partial_widgets, widgets) = Self::run_builders(
            path,
            first,
            &value_node,
            builders.iter().map(|builder| builder.as_ref()),
        );
        Self {
            key: first.clone(),
            widgets,
            partial_widgets,
            value: value_node,
        }
    }
    fn update_entry<'a>(
        &mut self,
        mut path: Vec<Key>,
        rest: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Result<()> {
        path.push(self.key.clone());
        self.value
            .update_entry(path.clone(), rest, value, builders)?;
        for widget in &mut self.widgets {
            widget.update_nt(&self.key, &self.value);
        }
        for builder_index in &self.partial_widgets {
            let builder = &builders[builder_index.index];
            match builder.create_kind(&self.key, &self.value) {
                BuildResult::Complete(kind) => self
                    .widgets
                    .push(Widget::new(Path::try_from(path.clone()).unwrap(), kind)),
                BuildResult::Partial(error) => {
                    event!(
                        Level::WARN,
                        "partial widget created at path {:?} with error {:?}",
                        self.key,
                        error
                    );
                }
                BuildResult::None => {}
            }
        }
        Ok(())
    }
    fn run_builders<'a>(
        mut path: Vec<Key>,
        key: &Key,
        value: &Value,
        builders: impl Iterator<Item = &'a dyn Builder>,
    ) -> (Vec<BuilderIndex>, Vec<Widget>) {
        path.push(key.clone());
        let mut partials = Vec::new();
        let mut widgets = Vec::new();
        for (i, builder) in builders.enumerate() {
            match builder.create_kind(key, value) {
                BuildResult::Complete(kind) => {
                    let title = Path::try_from(path.clone()).unwrap();
                    widgets.push(Widget::new(title, kind));
                }
                BuildResult::Partial(_) => partials.push(BuilderIndex { index: i }),
                BuildResult::None => {}
            }
        }
        (partials, widgets)
    }
    fn widgets<'a>(&'a self, widgets: &mut Vec<&'a Widget>) {
        widgets.extend(self.widgets.iter());
        if let Value::Branch(nodes) = &self.value {
            for node in &nodes.nodes {
                node.widgets(widgets);
            }
        }
    }
}

pub enum Value {
    Leaf(network_tables::Value),
    Branch(Nodes),
}

impl Value {
    pub fn new<'a>(
        path: Vec<Key>,
        mut keys: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Self {
        if let Some(first) = keys.next() {
            Self::Branch(Nodes {
                nodes: vec![Node::new(path, first, keys, value, builders)],
            })
        } else {
            Self::Leaf(value.clone())
        }
    }
    fn update_entry<'a>(
        &mut self,
        path: Vec<Key>,
        mut keys: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Result<()> {
        match (self, keys.next()) {
            (Value::Leaf(_), Some(_)) => Err(Error::ExpectedValue.into()),
            (Value::Leaf(old_value), None) => {
                *old_value = value.clone();
                Ok(())
            }
            (Value::Branch(nodes), Some(first)) => {
                nodes.update_entry(path, first, keys, value, builders)
            }
            (Value::Branch(_), None) => Err(Error::ExpectedBranch.into()),
        }
    }

    pub fn try_get_nodes(&self) -> Option<&Nodes> {
        match self {
            Value::Leaf(_) => None,
            Value::Branch(nodes) => Some(nodes),
        }
    }

    pub fn try_get_value(&self) -> Option<network_tables::Value> {
        match self {
            Value::Leaf(leaf) => Some(leaf.clone()),
            Value::Branch(_) => None,
        }
    }
}

#[derive(Default)]
pub struct Nodes {
    nodes: Vec<Node>,
}
impl Nodes {
    fn update_entry<'a>(
        &mut self,
        path: Vec<Key>,
        first: &Key,
        rest: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Result<()> {
        for node in &mut self.nodes {
            if &node.key == first {
                return node.update_entry(path, rest, value, builders);
            }
        }
        self.nodes
            .push(Node::new(path, first, rest, value, builders));
        Ok(())
    }

    pub fn try_get_value(&self, key: &str) -> Option<network_tables::Value> {
        for node in &self.nodes {
            if node.key == key {
                return node.value.try_get_value();
            }
        }
        None
    }
}

pub struct Tree {
    // represented as a boxed array rather than a vector to emphasize the fact that our builder indicies aren't protected by the type system
    builders: Box<[Box<dyn widgets::Builder>]>,
    nodes: Nodes,
}

impl Tree {
    pub fn new(builders: impl IntoIterator<Item = Box<dyn widgets::Builder>>) -> Self {
        let builders_vec: Vec<_> = builders.into_iter().collect();
        Self {
            builders: builders_vec.into_boxed_slice(),
            nodes: Nodes::default(),
        }
    }

    pub fn update_entry(&mut self, entry: &Entry) -> Result<()> {
        self.nodes.update_entry(
            Vec::new(),
            &entry.path.first,
            entry.path.rest.iter(),
            &entry.value,
            &self.builders,
        )
    }

    pub fn widgets(&self) -> Vec<&Widget> {
        let mut widgets = Vec::new();
        for node in &self.nodes.nodes {
            node.widgets(&mut widgets);
        }
        widgets
    }
}
