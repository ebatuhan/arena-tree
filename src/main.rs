fn main() {
    let mut tree = Tree::<String>::new();

    let s1 = String::from("A");
    let s2 = String::from("B");
    let s3 = String::from("B_A");
    let s4 = String::from("B_B");
    let s5 = String::from("B_C");
    let s6 = String::from("C");

    let i1 = tree.insert(s1);
    let i2 = tree.insert(s2);
    let i3 = tree.insert(s3);
    let i4 = tree.insert(s4);
    let i5 = tree.insert(s5);
    let i6 = tree.insert(s6);

    tree.set_parent(i5, i2);
    tree.set_parent(i4, i2);
    tree.set_parent(i3, i2);

    for value in tree.traverse_from(i2) {
        println!("{}", value.1);
    }
}

#[derive(Copy, Clone)]
struct NodeId(usize);

struct Node<T> {
    value: T,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn traverse_from(&self, root: NodeId) -> TreeTraversal<'_, T> {
        TreeTraversal {
            tree: self,
            stack: vec![root],
        }
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        Some(&mut self.nodes[id.0].value)
    }

    pub fn insert(&mut self, value: T) -> NodeId {
        let node = Node{ value, parent: None, children: Vec::new() };
        let id = NodeId(self.nodes.len());

        self.nodes.push(node);

        if id.0 != 0 {
            self.set_parent(id, NodeId(0));
        };

        id
    }

    pub fn set_parent(&mut self, id: NodeId, parent_id: NodeId) {
        let old_parent = { self.get_node_mut(id).unwrap().parent };

        if let Some(old_parent_id) = old_parent {
            self.get_node_mut(old_parent_id)
                .unwrap()
                .children
                .retain(|child_id| child_id.0 != id.0);
        }

        self.get_node_mut(id).unwrap().parent = Some(parent_id);
        self.get_node_mut(parent_id).unwrap().children.push(id);
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        Some(&self.nodes[id.0].value)
    }

    fn get_node(&self, id: NodeId) -> Option<&Node<T>> {
        Some(&self.nodes[id.0])
    }

    fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        Some(&mut self.nodes[id.0])
    }
}

struct TreeTraversal<'a, T> {
    tree: &'a Tree<T>,
    stack: Vec<NodeId>,
}

impl<'a, T> Iterator for TreeTraversal<'a, T> {
    type Item = (NodeId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let node_id = self.stack.pop()?;

        let node = self.tree.get_node(node_id).unwrap();

        for n in node.children.iter().rev() {
            self.stack.push(*n);
        }

        Some((node_id, &node.value))
    }
}
