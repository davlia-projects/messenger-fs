use std::collections::HashMap;

type NodeIdx = u64;

pub struct Node<T> {
    pub children: Vec<NodeIdx>,
    pub parent: Option<NodeIdx>,
    pub entry: T,
}

impl<T> Node<T> {
    pub fn new(parent: Option<NodeIdx>, entry: T) -> Self {
        Self {
            children: Vec::new(),
            parent,
            entry,
        }
    }
}

pub struct Tree<T> {
    arena: HashMap<NodeIdx, Node<T>>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self {
            arena: HashMap::new(),
        }
    }

    pub fn add(&mut self, parent: Option<NodeIdx>, idx: NodeIdx, entry: T) {
        let node = Node::new(parent, entry);
        self.arena.insert(idx, node);
        if let Some(parent) = parent {
            let mut parent_node = self.arena.get_mut(&parent).expect("Found orphaned node");
            parent_node.children.push(idx);
        }
    }

    pub fn get(&mut self, idx: NodeIdx) -> Option<&mut Node<T>> {
        self.arena.get_mut(&idx)
    }

    pub fn delete(&mut self, parent: Option<NodeIdx>, idx: NodeIdx) {
        self.arena.remove(&idx);
        if let Some(parent) = parent {
            if let Some(mut node) = self.arena.get_mut(&parent) {
                node.children.retain(|&child| child != idx);
            }
        }
    }
}
