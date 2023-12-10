use std::fmt;

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
    #[error("Expected a value at {0:?}")]
    ExpectedValue(Vec<Key>),
    #[error("Expected a branch at {0:?} instead got value {1}")]
    ExpectedBranch(Vec<Key>, network_tables::Value),
    #[error("Expected entry {1:?}, {2:?} at path {0:?}")]
    NoSuchEntry(Vec<String>, String, Vec<String>),
}

#[derive(Copy, Clone, Debug)]
pub struct BuilderIndex {
    pub index: usize,
}

pub struct Node {
    pub key: Key,
    pub widgets: Vec<Widget>,
    pub partial_widgets: Vec<BuilderIndex>,
    pub value: Value,
}

impl Node {
    fn widgets<'a>(&'a self, widgets: &mut Vec<&'a Widget>) {
        for widget in &self.widgets {
            widgets.push(widget);
        }
        self.value.widgets(widgets);
    }

    fn update_entry(
        &mut self,
        mut path: Vec<Key>,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        path.push(self.key.clone());

        self.value
            .update_entry(path.clone(), rest, value, builders)?;

        for widget in self.widgets.iter_mut() {
            widget.update_nt(&self.key, &self.value);
        }

        let (widgets, partials) = Self::run_builders(
            &self.key,
            &self.value,
            path,
            builders,
            &self.partial_widgets,
        );

        if !widgets.is_empty() {
            self.value.strip_widgets();
        }

        self.widgets.extend(widgets);
        self.partial_widgets = partials;

        Ok(())
    }

    fn run_builders(
        key: &Key,
        value: &Value,
        path: Vec<Key>,
        builders: &[Box<dyn Builder>],
        partial_widgets: &[BuilderIndex],
    ) -> (Vec<Widget>, Vec<BuilderIndex>) {
        let mut widgets = Vec::new();
        let mut partials = Vec::new();
        for index in partial_widgets {
            let builder = builders[index.index].as_ref();

            match builder.create_kind(key, value) {
                BuildResult::Complete(kind) => {
                    widgets.push(Widget::new(Path::try_from(path.clone()).unwrap(), kind));
                }
                BuildResult::Partial(error) => {
                    partials.push(*index);
                }
                BuildResult::None => {}
            }
        }
        (widgets, partials)
    }

    fn from_entry(
        mut path: Vec<Key>,
        first: &Key,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<Self> {
        path.push(first.clone());
        let mut node_value = if rest.is_empty() {
            Value::Leaf(value)
        } else {
            let mut branches = Nodes::default();
            branches.create_entry(path.clone(), &rest[0], &rest[1..], value, builders)?;
            Value::Branch(branches)
        };
        let (widgets, partial_widgets) = Self::run_builders(
            first,
            &node_value,
            path,
            builders,
            &(0..builders.len())
                .map(|index| BuilderIndex { index })
                .collect::<Vec<_>>(),
        );
        if !widgets.is_empty() {
            node_value.strip_widgets();
        }
        Ok(Self {
            key: first.clone(),
            widgets,
            partial_widgets,
            value: node_value,
        })
    }

    fn create_entry(
        &mut self,
        mut path: Vec<Key>,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        path.push(self.key.clone());

        self.value
            .create_entry(path.clone(), rest, value, builders)?;

        let (widgets, partial_widgets) = Self::run_builders(
            &self.key,
            &self.value,
            path,
            builders,
            &(0..builders.len())
                .map(|index| BuilderIndex { index })
                .collect::<Vec<_>>(),
        );

        if !widgets.is_empty() {
            self.value.strip_widgets();
        }

        self.widgets.extend(widgets);
        self.partial_widgets = partial_widgets;

        Ok(())
    }

    fn strip_widgets(&mut self) {
        self.widgets.clear();
        self.value.strip_widgets();
    }
}

pub enum Value {
    Leaf(network_tables::Value),
    Branch(Nodes),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Self::Leaf(value) = self {
            write!(f, "{value}")
        } else {
            write!(f, "[branch]")
        }
    }
}

impl Value {
    fn update_entry(
        &mut self,
        path: Vec<Key>,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        match (self, rest) {
            (Self::Leaf(old_value), []) => {
                *old_value = value;
                return Ok(());
            }
            (Self::Leaf(old_value), [first, rest @ ..]) => {
                Err(Error::ExpectedBranch(path, old_value.clone()).into())
            }
            (Self::Branch(nodes), []) => Err(Error::ExpectedValue(path).into()),
            (Self::Branch(nodes), [first, rest @ ..]) => {
                nodes.update_entry(path, first, rest, value, builders)
            }
        }
    }

    fn widgets<'a>(&'a self, widgets: &mut Vec<&'a Widget>) {
        if let Value::Branch(nodes) = self {
            nodes.widgets(widgets);
        }
    }

    pub fn try_get_nodes(&self) -> Option<&Nodes> {
        if let Self::Branch(nodes) = self {
            Some(nodes)
        } else {
            None
        }
    }

    fn create_entry(
        &mut self,
        path: Vec<Key>,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        match (self, rest) {
            (Self::Leaf(old_value), []) => {
                *old_value = value;
                Ok(())
            }
            (Self::Leaf(old_value), [first, rest @ ..]) => {
                Err(Error::ExpectedBranch(path, old_value.clone()).into())
            }
            (Self::Branch(nodes), []) => Err(Error::ExpectedValue(path).into()),
            (Self::Branch(nodes), [first, tail @ ..]) => {
                nodes.create_entry(path, first, tail, value, builders)
            }
        }
    }

    fn strip_widgets(&mut self) {
        if let Self::Branch(nodes) = self {
            nodes.strip_widgets();
        }
    }
}

#[derive(Default)]
pub struct Nodes {
    nodes: Vec<Node>,
}

impl Nodes {
    fn update_entry(
        &mut self,
        path: Vec<Key>,
        first: &Key,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        for node in &mut self.nodes {
            if &node.key == first {
                return node.update_entry(path, rest, value, builders);
            }
        }
        Err(Error::NoSuchEntry(path, first.clone(), rest.to_vec()).into())
    }

    fn create_entry(
        &mut self,
        path: Vec<Key>,
        first: &Key,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        for node in &mut self.nodes {
            if &node.key == first {
                return node.create_entry(path, rest, value, builders);
            }
        }
        let node = Node::from_entry(path, first, rest, value, builders)?;
        self.nodes.push(node);
        Ok(())
    }

    fn widgets<'a>(&'a self, widgets: &mut Vec<&'a Widget>) {
        for node in &self.nodes {
            node.widgets(widgets);
        }
    }

    pub fn try_get_value(&self, key: impl Into<Key>) -> Option<network_tables::Value> {
        let key = key.into();
        for node in &self.nodes {
            if &node.key == &key {
                if let Value::Leaf(value) = &node.value {
                    return Some(value.clone());
                }
            }
        }
        None
    }

    fn strip_widgets(&mut self) {
        for node in &mut self.nodes {
            node.strip_widgets();
        }
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
        event!(Level::INFO, "updating tree with entry {entry:?}");
        self.nodes.update_entry(
            Vec::new(),
            &entry.path.first,
            &entry.path.rest,
            entry.value.clone(),
            &self.builders,
        )
    }

    pub fn create_entry(&mut self, entry: &Entry) -> Result<()> {
        self.nodes.create_entry(
            Vec::new(),
            &entry.path.first,
            &entry.path.rest,
            entry.value.clone(),
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
