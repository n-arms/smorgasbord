use std::fmt;

use crate::{
    backend::{Entry, Key, Path},
    widgets::Widget,
    widgets::{self, BuildResult, Builder},
};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Expected a value at {0:?}")]
    ExpectedValue(Vec<Key>),
    #[error("Expected a branch at {0:?} instead got value {1}")]
    ExpectedBranch(Vec<Key>, network_tables::Value),
    #[error("Expected entry {1:?}, {2:?} at path {0:?}")]
    NoSuchEntry(Vec<Key>, String, Vec<Key>),
}

#[derive(Copy, Clone)]
pub struct BuilderIndex {
    pub index: usize,
}

impl fmt::Debug for BuilderIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.index.fmt(f)
    }
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
        path: &mut Vec<Key>,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        path.push(self.key.clone());

        if let Err(error) = self.value.update_entry(path, rest, value, builders) {
            path.pop();
            return Err(error);
        }

        for widget in &mut self.widgets {
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

        path.pop();

        Ok(())
    }

    fn run_builders(
        key: &Key,
        value: &Value,
        path: &[Key],
        builders: &[Box<dyn Builder>],
        partial_widgets: &[BuilderIndex],
    ) -> (Vec<Widget>, Vec<BuilderIndex>) {
        let mut widgets = Vec::new();
        let mut partials = Vec::new();
        for index in partial_widgets {
            let builder = builders[index.index].as_ref();

            match builder.create_kind(key, value) {
                BuildResult::Complete(kind) => {
                    widgets.push(Widget::new(Path::try_from(path.to_owned()).unwrap(), kind));
                }
                BuildResult::Partial(_) => {
                    partials.push(*index);
                }
                BuildResult::None => {}
            }
        }
        (widgets, partials)
    }

    fn from_entry(
        path: &mut Vec<Key>,
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
            if let Err(error) = branches.create_entry(path, &rest[0], &rest[1..], value, builders) {
                path.pop();
                return Err(error);
            }
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
        path.pop();
        Ok(Self {
            key: first.clone(),
            widgets,
            partial_widgets,
            value: node_value,
        })
    }

    fn create_entry(
        &mut self,
        path: &mut Vec<Key>,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        path.push(self.key.clone());

        if let Err(error) = self.value.create_entry(path, rest, value, builders) {
            path.pop();
            return Err(error);
        }

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

        path.pop();

        Ok(())
    }

    fn strip_widgets(&mut self) {
        self.widgets.clear();
        self.value.strip_widgets();
    }

    fn get<'a>(
        &self,
        rest: impl ExactSizeIterator<Item = &'a Key> + fmt::Debug,
    ) -> Option<&Widget> {
        if rest.len() == 0 {
            return self.widgets.first();
        }
        self.value.get(rest)
    }

    fn get_mut<'a>(&mut self, rest: impl ExactSizeIterator<Item = &'a Key>) -> Option<&mut Widget> {
        if rest.len() == 0 {
            return self.widgets.first_mut();
        }
        self.value.get_mut(rest)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]: {:?}", self.key, self.widgets.len(), self.value)
    }
}

pub enum Value {
    Leaf(network_tables::Value),
    Branch(Nodes),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Leaf(value) => write!(f, "{value}"),
            Self::Branch(nodes) => nodes.fmt(f),
        }
    }
}

impl Value {
    fn update_entry(
        &mut self,
        path: &mut Vec<Key>,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        match (self, rest) {
            (Self::Leaf(old_value), []) => {
                *old_value = value;
                Ok(())
            }
            (Self::Leaf(old_value), [_, ..]) => {
                Err(Error::ExpectedBranch(path.clone(), old_value.clone()).into())
            }
            (Self::Branch(_), []) => Err(Error::ExpectedValue(path.clone()).into()),
            (Self::Branch(nodes), [first, rest @ ..]) => {
                nodes.update_entry(path, first, rest, value, builders)
            }
        }
    }

    fn widgets<'a>(&'a self, widgets: &mut Vec<&'a Widget>) {
        if let Self::Branch(nodes) = self {
            nodes.widgets(widgets);
        }
    }

    pub const fn try_get_nodes(&self) -> Option<&Nodes> {
        if let Self::Branch(nodes) = self {
            Some(nodes)
        } else {
            None
        }
    }

    fn create_entry(
        &mut self,
        path: &mut Vec<Key>,
        rest: &[Key],
        value: network_tables::Value,
        builders: &[Box<dyn Builder>],
    ) -> Result<()> {
        match (self, rest) {
            (Self::Leaf(old_value), []) => {
                *old_value = value;
                Ok(())
            }
            (Self::Leaf(old_value), [_, ..]) => {
                Err(Error::ExpectedBranch(path.clone(), old_value.clone()).into())
            }
            (Self::Branch(_), []) => Err(Error::ExpectedValue(path.clone()).into()),
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

    pub fn try_get_value(&self) -> Option<network_tables::Value> {
        if let Self::Leaf(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }

    fn get<'a>(
        &self,
        mut rest: impl ExactSizeIterator<Item = &'a Key> + fmt::Debug,
    ) -> Option<&Widget> {
        match (self, rest.next()) {
            (Self::Branch(nodes), Some(first)) => nodes.get(first, rest),
            _ => None,
        }
    }

    fn get_mut<'a>(
        &mut self,
        mut rest: impl ExactSizeIterator<Item = &'a Key>,
    ) -> Option<&mut Widget> {
        match (self, rest.next()) {
            (Self::Branch(nodes), Some(first)) => nodes.get_mut(first, rest),
            _ => None,
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
        path: &mut Vec<Key>,
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
        Err(Error::NoSuchEntry(path.clone(), first.to_string(), rest.to_vec()).into())
    }

    fn create_entry(
        &mut self,
        path: &mut Vec<Key>,
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
            if node.key == key {
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

    pub fn iter(&self) -> impl Iterator<Item = &'_ Node> {
        self.nodes.iter()
    }

    fn get<'a>(
        &self,
        first: &Key,
        rest: impl ExactSizeIterator<Item = &'a Key> + fmt::Debug,
    ) -> Option<&Widget> {
        for node in &self.nodes {
            if &node.key == first {
                return node.get(rest);
            }
        }
        None
    }

    fn get_mut<'a>(
        &mut self,
        first: &Key,
        rest: impl ExactSizeIterator<Item = &'a Key>,
    ) -> Option<&mut Widget> {
        for node in &mut self.nodes {
            if &node.key == first {
                return node.get_mut(rest);
            }
        }
        None
    }
}

impl fmt::Debug for Nodes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut set = f.debug_set();

        for node in &self.nodes {
            set.entry(node);
        }

        set.finish()
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
            &mut Vec::new(),
            &entry.path.first,
            &entry.path.rest,
            entry.value,
            &self.builders,
        )
    }

    pub fn create_entry(&mut self, entry: Entry) -> Result<()> {
        self.nodes.create_entry(
            &mut Vec::new(),
            &entry.path.first,
            &entry.path.rest,
            entry.value,
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

    pub fn get(&self, path: &Path) -> Option<&Widget> {
        self.nodes.get(&path.first, path.rest.iter())
    }

    pub fn get_mut(&mut self, path: &Path) -> Option<&mut Widget> {
        self.nodes.get_mut(&path.first, path.rest.iter())
    }
}

impl fmt::Debug for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.nodes.fmt(f)
    }
}
