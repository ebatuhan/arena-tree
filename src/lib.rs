use std::{array, mem::take};

use crate::node_id::NodeId;

mod node_id;

#[derive(Debug)]
struct Node<T> {
    value: T,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    generation: i32,
}

#[derive(Debug)]
pub struct Tree<T> {
    nodes: Vec<Option<Node<T>>>,
    free_slots: Vec<NodeId>,
    root: Option<NodeId>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free_slots: Vec::new(),
            root: None,
        }
    }

    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn traverse(&self, root: NodeId) -> impl Iterator<Item = (NodeId, &T)> + '_ {
        self.traverse_nodes(root)
            .map(|(id, node)| (id, &node.value))
    }

    pub fn traverse_ids(&self, root: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.traverse_nodes(root).map(|(id, _)| id)
    }

    fn traverse_nodes(&self, root: NodeId) -> TreeNodeTraversal<'_, T> {
        TreeNodeTraversal {
            stack: vec![root],
            tree: self,
        }
    }

    pub fn insert(&mut self, value: T) -> NodeId {
        let id = {
            if let Some(slot_id) = self.free_slots.pop() {
                slot_id
            } else {
                NodeId::new(self.nodes.len())
            }
        };

        let node = Node {
            value,
            parent: None,
            children: Vec::new(),
            generation: id.generation(),
        };

        if id.index() == self.nodes.len() {
            self.nodes.push(Some(node));
        } else {
            self.nodes[id.index()] = Some(node);
        }

        if let Some(root) = self.root {
            self.move_node(id, root, Placement::In);
        } else {
            self.root = Some(id);
        }

        id
    }

    fn detach(&mut self, id: NodeId) {
        self.nodes[id.index()] = None;
        self.free_slots.push(id.next_gen());
    }

    pub fn remove_node(&mut self, root_id: NodeId) {
        if Some(root_id) == self.root {
            return;
        }

        if let Some(parent_id) = self.get_node(root_id).unwrap().parent {
            self.get_node_mut(parent_id)
                .unwrap()
                .children
                .retain(|child_id| *child_id != root_id);
        }

        let mut nodes_to_remove = Vec::<NodeId>::new();

        for node_id in self.traverse_ids(root_id) {
            nodes_to_remove.push(node_id);
        }

        for node_id in nodes_to_remove {
            self.detach(node_id);
        }
    }

    fn is_ancestor(&self, id: NodeId, mut target_id: NodeId) -> bool {
        loop {
            if id == target_id {
                return true;
            }

            let Some(node) = self.get_node(target_id) else {
                return false;
            };

            let Some(parent_id) = node.parent else {
                return false;
            };

            target_id = parent_id
        }
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.nodes
            .get(id.index())?
            .as_ref()
            .filter(|node| node.generation == id.generation())
            .map(|node| &node.value)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.nodes
            .get_mut(id.index())?
            .as_mut()
            .filter(|node| node.generation == id.generation())
            .map(|node| &mut node.value)
    }

    fn get_node(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes
            .get(id.index())?
            .as_ref()
            .filter(|node| node.generation == id.generation())
    }

    fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes
            .get_mut(id.index())?
            .as_mut()
            .filter(|node| node.generation == id.generation())
    }

    pub fn move_node(&mut self, id: NodeId, target_id: NodeId, placement: Placement) {
        match placement {
            Placement::In => {
                self.place_node_under(id, target_id, None);
            }
            Placement::Before => {
                self.place_node_next_to(id, target_id, 0);
            }
            Placement::After => {
                self.place_node_next_to(id, target_id, 1);
            }
        }
    }

    fn place_node_next_to(&mut self, id: NodeId, target_id: NodeId, offset: usize) {
        let Some(parent_id) = self.parent(target_id) else {
            return;
        };

        if id == target_id
            || Some(id) == self.root
            || !self.contains(id)
            || !self.contains(target_id)
            || !self.contains(parent_id)
            || self.is_ancestor(id, parent_id)
        {
            return;
        }

        self.detach_from_parent(id);

        let Some(target_position) = self.find_child_index(target_id) else {
            return;
        };

        self.get_node_mut(id).unwrap().parent = Some(parent_id);

        self.get_node_mut(parent_id)
            .unwrap()
            .children
            .insert(target_position + offset, id);
    }

    fn place_node_under(&mut self, id: NodeId, parent_id: NodeId, index: Option<usize>) {
        if id == parent_id
            || Some(id) == self.root
            || !self.contains(id)
            || !self.contains(parent_id)
            || self.is_ancestor(id, parent_id)
        {
            return;
        }

        self.detach_from_parent(id);
        self.get_node_mut(id).unwrap().parent = Some(parent_id);

        let parent = self.get_node_mut(parent_id).unwrap();

        match index {
            Some(index) => parent.children.insert(index, id),
            None => parent.children.push(id),
        }
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.get_node(id).is_some()
    }

    fn find_child_index(&self, id: NodeId) -> Option<usize> {
        self.parent_node(id)
            .and_then(|node| node.children.iter().position(|node_id| node_id == &id))
    }

    fn detach_from_parent(&mut self, id: NodeId) {
        let Some(old_parent_id) = self.get_node(id).and_then(|node| node.parent) else {
            return;
        };

        if let Some(old_parent) = self.get_node_mut(old_parent_id) {
            old_parent.children.retain(|child_id| *child_id != id);
        }

        if let Some(node) = self.get_node_mut(id) {
            node.parent = None;
        }
    }

    fn parent_node(&self, id: NodeId) -> Option<&Node<T>> {
        self.get_node(id)
            .and_then(|node| node.parent)
            .and_then(|parent_id| self.get_node(parent_id))
    }

    fn children_nodes(&self, id: NodeId) -> Vec<&Node<T>> {
        self.get_node(id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|id| self.get_node(*id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id).and_then(|node| node.parent)
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.get_node(id)
            .map(|node| node.children.as_slice())
            .unwrap_or(&[])
    }
}

pub enum Placement {
    After,
    Before,
    In,
}

struct TreeNodeTraversal<'a, T> {
    stack: Vec<NodeId>,
    tree: &'a Tree<T>,
}

impl<'a, T> Iterator for TreeNodeTraversal<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        let node_id = self.stack.pop()?;

        let node = self.tree.get_node(node_id)?;

        for child in node.children.iter().rev() {
            self.stack.push(*child);
        }

        Some((node_id, node))
    }
}
