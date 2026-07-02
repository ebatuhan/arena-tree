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

        let  node = Node {
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
            self.set_parent(id, root);
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
        let node = self.get_node(id);

        if let Some(node) = node {
            Some(&node.value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let node = self.get_node_mut(id);

        if let Some(node) = node {
            Some(&mut node.value)
        } else {
            None
        }
    }

    fn get_node(&self, id: NodeId) -> Option<&Node<T>> {
        let node = self.nodes.get(id.index())?.as_ref()?;

        if node.generation == id.generation() {
            Some(node)
        } else {
            None
        }
    }

    fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        let node = self.nodes.get_mut(id.index())?.as_mut()?;
        if node.generation == id.generation() {
            Some(node)
        } else {
            None
        }
    }

    pub fn set_parent(&mut self, id: NodeId, parent_id: NodeId) {
        if id == parent_id {
            return;
        }

        if Some(id) == self.root {
            return;
        }

        if self.get_node(id).is_none() || self.get_node(parent_id).is_none() {
            return;
        }

        if self.is_ancestor(id, parent_id) {
            return;
        }

        if let Some(old_parent_id) = self.get_node(id).unwrap().parent {
            self.get_node_mut(old_parent_id)
                .unwrap()
                .children
                .retain(|child_id| *child_id != id);
        };

        self.get_node_mut(id).unwrap().parent = Some(parent_id);
        self.get_node_mut(parent_id).unwrap().children.push(id);
    }
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
