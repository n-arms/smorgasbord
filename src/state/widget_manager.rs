use network_tables::Value;

use crate::trie::Keys;
use crate::widgets::{Builder, Widget};
use crate::{
    nt::Key,
    trie::{Node, NodeValue, Nodes, Trie},
};

#[derive(Default)]
pub struct WidgetManager {
    builders: Vec<Box<dyn Builder>>,
}

impl WidgetManager {
    #[allow(dead_code)]
    pub fn new(builders: Vec<Box<dyn Builder>>) -> Self {
        Self { builders }
    }

    pub fn with(mut self, builder: impl Builder + 'static) -> Self {
        self.builders.push(Box::new(builder));
        self
    }

    #[allow(dead_code)]
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
            /*
                let Some(kind) = builder.create_kind(data) else {
                    continue;
                };
            */
            let kind = todo!();
            let path = Keys::from_vec(prefix).unwrap();
            let widget = Widget::new(path, kind);
            output.push(widget);
            return;
        }

        if let NodeValue::Branch(nodes) = &data.value {
            self.visit_nodes(nodes, prefix, output);
        }
    }
}
