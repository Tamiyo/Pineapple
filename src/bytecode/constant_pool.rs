use std::collections::HashMap;
use crate::bytecode::Constant;

#[derive(Debug, Clone)]
pub struct ConstantPool {
    pub pool: Vec<Constant>,
    pub cache: HashMap<Constant, usize>,
}

impl ConstantPool {
    pub fn new() -> Self {
        ConstantPool {
            pool: vec![],
            cache: HashMap::<Constant, usize>::new(),
        }
    }

    pub fn insert(&mut self, constant: Constant) -> usize {
        if self.cache.contains_key(&constant) {
            *self.cache.get(&constant).unwrap()
        } else {
            self.cache.insert(constant.clone(), self.pool.len());
            self.pool.push(constant);
            self.pool.len() - 1
        }
    }

    pub fn get(&self, constant_index: usize) -> &Constant {
        &self.pool[constant_index]
    }
}
