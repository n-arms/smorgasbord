#![allow(dead_code)]

use std::{fmt, mem::swap};

#[derive(Clone)]
pub struct Nodes<K, V> {
    pub nodes: Vec<Node<K, V>>,
}

#[derive(Clone)]
pub struct Node<K, V> {
    pub key: K,
    pub value: NodeValue<K, V>,
}

#[derive(Clone)]
pub enum NodeValue<K, V> {
    Leaf(V),
    Branch(Nodes<K, V>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keys<K, I> {
    pub first: K,
    pub rest: I,
}

#[derive(Debug)]
pub struct KeysRef<'a, K, I> {
    pub first: &'a K,
    pub rest: I,
}

impl<K> Keys<K, Vec<K>> {
    pub fn from_vec(items: Vec<K>) -> Option<Self> {
        let mut rest = items;
        let first = rest.remove(0);
        Some(Keys { first, rest })
    }

    pub fn push(&mut self, arg: impl Into<K>) {
        self.rest.push(arg.into());
    }
}

impl<'a, K, I: Iterator<Item = &'a K>> KeysRef<'a, K, I> {
    pub fn new<T: IntoIterator<Item = &'a K, IntoIter = I>>(items: T) -> Option<Self> {
        let mut rest = items.into_iter();
        let first = rest.next()?;
        Some(KeysRef { first, rest })
    }
}

pub struct Trie<K, V> {
    pub root: Nodes<K, V>,
}

impl<K, V> Trie<K, V> {
    pub fn new() -> Self {
        Self {
            root: Nodes { nodes: Vec::new() },
        }
    }
}

impl<K: Clone, V> Trie<K, V> {
    pub fn walk(&self, func: &mut impl FnMut(&[K], &V)) {
        self.root.walk(Vec::new(), func);
    }
}

pub enum TrieError<K, V> {
    ExpectedBranch(V),
    ExpectedValue(Nodes<K, V>),
}

impl<K: Eq + Clone + fmt::Debug, V: Clone + fmt::Debug + fmt::Display> Trie<K, V> {
    pub fn insert<I: IntoIterator<Item = K>>(
        &mut self,
        keys: Keys<K, I>,
        value: V,
    ) -> Result<Option<V>, TrieError<K, V>> {
        self.root.insert(keys.first, keys.rest.into_iter(), value)
    }

    pub fn get<'a, I: Iterator<Item = &'a K>>(
        &self,
        keys: KeysRef<'a, K, I>,
    ) -> Result<Option<&V>, TrieError<K, V>>
    where
        K: 'a,
    {
        self.root.get(keys.first, keys.rest)
    }

    pub fn get_subtrie<'a, I: Iterator<Item = &'a K> + fmt::Debug>(
        &self,
        keys: KeysRef<'a, K, I>,
    ) -> Option<&Node<K, V>>
    where
        K: 'a,
    {
        self.root.get_subtrie(keys.first, keys.rest)
    }
}

impl<K: Clone, V> Nodes<K, V> {
    fn walk(&self, prefix: Vec<K>, func: &mut impl FnMut(&[K], &V)) {
        for node in &self.nodes {
            node.walk(prefix.clone(), func);
        }
    }
}

impl<K: Eq + Clone + fmt::Debug, V: Clone + fmt::Debug + fmt::Display> Nodes<K, V> {
    fn insert(
        &mut self,
        first: K,
        keys: impl Iterator<Item = K>,
        value: V,
    ) -> Result<Option<V>, TrieError<K, V>> {
        for node in self.nodes.iter_mut() {
            if node.key == first {
                return node.value.insert(keys, value);
            }
        }
        self.nodes.push(Node::new(first, keys, value));
        Ok(None)
    }

    fn get<'a>(
        &self,
        first: &'a K,
        keys: impl Iterator<Item = &'a K>,
    ) -> Result<Option<&V>, TrieError<K, V>>
    where
        K: 'a,
    {
        for node in self.nodes.iter() {
            if &node.key == first {
                return node.value.get(keys);
            }
        }
        Ok(None)
    }

    fn get_subtrie<'a>(
        &self,
        first: &'a K,
        mut rest: impl Iterator<Item = &'a K> + fmt::Debug,
    ) -> Option<&Node<K, V>>
    where
        K: 'a,
    {
        for node in self.nodes.iter() {
            if &node.key == first {
                return match &node.value {
                    NodeValue::Leaf(_) => {
                        if rest.next().is_none() {
                            Some(node)
                        } else {
                            None
                        }
                    }
                    NodeValue::Branch(branches) => {
                        if let Some(next) = rest.next() {
                            branches.get_subtrie(next, rest)
                        } else {
                            Some(node)
                        }
                    }
                };
            }
        }
        None
    }
}

impl<K: Clone, V> Node<K, V> {
    fn walk(&self, mut prefix: Vec<K>, func: &mut impl FnMut(&[K], &V)) {
        prefix.push(self.key.clone());
        match &self.value {
            NodeValue::Leaf(value) => func(&prefix, value),
            NodeValue::Branch(branches) => branches.walk(prefix, func),
        }
    }
}

impl<K: Eq + Clone + fmt::Debug, V: Clone + fmt::Debug + fmt::Display> Node<K, V> {
    fn new(first: K, keys: impl Iterator<Item = K>, value: V) -> Self {
        Self {
            key: first,
            value: NodeValue::new(keys, value),
        }
    }
}

impl<K: Eq + Clone + fmt::Debug, V: Clone + fmt::Display + fmt::Debug> NodeValue<K, V> {
    fn new(mut keys: impl Iterator<Item = K>, value: V) -> Self {
        match keys.next() {
            Some(key) => Self::Branch(Nodes {
                nodes: vec![Node::new(key, keys, value)],
            }),
            None => Self::Leaf(value),
        }
    }

    fn insert(
        &mut self,
        mut keys: impl Iterator<Item = K>,
        mut value: V,
    ) -> Result<Option<V>, TrieError<K, V>> {
        match self {
            NodeValue::Leaf(old_value) => {
                if keys.next().is_some() {
                    Err(TrieError::ExpectedBranch(old_value.clone()))
                } else {
                    swap(old_value, &mut value);
                    Ok(Some(value))
                }
            }
            NodeValue::Branch(branches) => {
                if let Some(key) = keys.next() {
                    branches.insert(key, keys, value)
                } else {
                    Err(TrieError::ExpectedValue(branches.clone()))
                }
            }
        }
    }

    fn get<'a>(&self, mut keys: impl Iterator<Item = &'a K>) -> Result<Option<&V>, TrieError<K, V>>
    where
        K: 'a,
    {
        match self {
            NodeValue::Leaf(value) => {
                if keys.next().is_some() {
                    Err(TrieError::ExpectedBranch(value.clone()))
                } else {
                    Ok(Some(value))
                }
            }
            NodeValue::Branch(branches) => {
                if let Some(key) = keys.next() {
                    branches.get(key, keys)
                } else {
                    Err(TrieError::ExpectedValue(branches.clone()))
                }
            }
        }
    }

    pub fn try_get_nodes(&self) -> Option<&Nodes<K, V>> {
        match self {
            NodeValue::Leaf(_) => None,
            NodeValue::Branch(nodes) => Some(nodes),
        }
    }
}

impl<K: fmt::Debug, V: fmt::Display> fmt::Debug for Trie<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.root.fmt(f)
    }
}

impl<K: fmt::Debug, V: fmt::Display> fmt::Debug for Nodes<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.nodes.iter().map(|node| (&node.key, &node.value)))
            .finish()
    }
}

impl<K: fmt::Debug, V: fmt::Display> fmt::Debug for Node<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entry(&self.key, &self.value).finish()
    }
}

impl<K: fmt::Debug, V: fmt::Display> fmt::Debug for NodeValue<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeValue::Leaf(value) => write!(f, "{}", value),
            NodeValue::Branch(branches) => branches.fmt(f),
        }
    }
}

impl<K: fmt::Debug, V: fmt::Display> fmt::Debug for TrieError<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrieError::ExpectedBranch(value) => write!(f, "{}", value),
            TrieError::ExpectedValue(branches) => branches.fmt(f),
        }
    }
}
