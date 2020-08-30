use std::rc::Rc;

// I don't even think we need this but I'll keep it in here just in case
pub struct Node<T> {
    inner: T,
    children: Vec<Rc<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(inner: T) -> Self {
        Node {
            inner,
            children: vec![],
        }
    }

    pub fn add_child(&mut self, child: T) {
        let node = Node::new(child);
        self.children.push(Rc::new(node));
    }
}

pub struct Tree<T> {
    root: Option<Node<T>>,
}
