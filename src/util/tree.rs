use std::rc::Rc;

pub struct Node<T> {
    inner: T,
    children: Vec<Rc<Node<T>>>,
}

pub struct Tree<T> {
    root: Option<Node<T>>,
}
