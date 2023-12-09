use crate::{
    nt::{Entry, Key},
    trie::Keys,
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

pub struct NodeInner {
    pub key: Key,
    pub value: Value,
}

impl Node {
    pub fn new<'a>(
        first: &Key,
        mut rest: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Self {
        let mut value_node = Value::new(rest, value, builders);
        Self {
            key: first.clone(),
            widgets: todo!(),
            partial_widgets: todo!(),
            value,
        }
    }
    pub fn update_entry<'a>(
        &mut self,
        first: &Key,
        mut rest: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Option<Result<()>> {
        if first == &self.key {
            if let Err(error) = self.value.update_entry(rest, value, builders) {
                return Some(Err(error));
            }
            for widget in &mut self.widgets {
                widget.update_nt(&mut self.key, &mut self.value);
            }
            for builder_index in &self.partial_widgets {
                let builder = &builders[builder_index.index];
                match builder.create_kind(&self) {
                    BuildResult::Complete(kind) => self.widgets.push(Widget::new(
                        Keys {
                            first: first.clone(),
                            rest: Vec::new(),
                        },
                        kind,
                    )),
                    BuildResult::Partial(error) => {
                        event!(Level::WARN, "partial widget created with error {:?}", error)
                    }
                    BuildResult::None => {}
                }
            }
            Some(Ok(()))
        } else {
            None
        }
    }
    fn run_builders<'a>(
        &self,
        builders: impl Iterator<Item = &'a dyn Builder>,
    ) -> (Vec<BuilderIndex>, Vec<Widget>) {
        let mut partials = Vec::new();
        let mut widgets = Vec::new();
        for (i, builder) in builders.enumerate() {
            match builder.create_kind(self) {
                BuildResult::Complete(kind) => {
                    let fake_title = Keys {
                        first: self.key.clone(),
                        rest: Vec::new(),
                    };
                    widgets.push(Widget {
                        title: fake_title,
                        value: kind,
                    });
                }
                BuildResult::Partial(error) => partials.push(BuilderIndex { index: i }),
                BuildResult::None => {}
            }
        }
        (partials, widgets)
    }
}

pub enum Value {
    Leaf(network_tables::Value),
    Branch(Nodes),
}

impl Value {
    pub fn new<'a>(
        mut keys: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Self {
        if let Some(first) = keys.next() {
            Self::Branch(Nodes {
                nodes: vec![Node::new(first, keys, value, builders)],
            })
        } else {
            Self::Leaf(value.clone())
        }
    }
    pub fn update_entry<'a>(
        &mut self,
        mut keys: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Result<()> {
        match (self, keys.next()) {
            (Value::Leaf(value), Some(rest)) => Err(Error::ExpectedValue.into()),
            (Value::Leaf(old_value), None) => {
                *old_value = value.clone();
                Ok(())
            }
            (Value::Branch(nodes), Some(first)) => nodes.update_entry(first, keys, value, builders),
            (Value::Branch(nodes), None) => Err(Error::ExpectedBranch.into()),
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
        first: &Key,
        rest: impl Iterator<Item = &'a Key>,
        value: &network_tables::Value,
        builders: &[Box<dyn widgets::Builder>],
    ) -> Result<()> {
        for node in &mut self.nodes {
            if &node.key == first {
                return node.value.update_entry(rest, value, builders);
            }
        }
        self.nodes.push(Node::new(first, rest, value, builders));
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

    pub fn update_entry(&mut self, entry: Entry) -> Result<()> {
        self.nodes.update_entry(
            &entry.path.first,
            entry.path.rest.iter(),
            &entry.value,
            &self.builders,
        )
    }
}
