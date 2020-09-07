use std::collections::HashMap;
use crate::core::value::Value;

#[derive(Debug, Clone)]
pub struct ConstantPool {
    pub pool: Vec<Value>,
    pub cache: HashMap<Value, usize>,
}

impl ConstantPool {
    pub fn new() -> Self {
        ConstantPool {
            pool: vec![],
            cache: HashMap::<Value, usize>::new(),
        }
    }

    pub fn insert(&mut self, value: Value) -> usize {
        if self.cache.contains_key(&value) {
            *self.cache.get(&value).unwrap()
        } else {
            self.cache.insert(value.clone(), self.pool.len());
            self.pool.push(value);
            self.pool.len() - 1
        }
    }

    pub fn get(&self, value_index: usize) -> &Value {
        &self.pool[value_index]
    }
}
