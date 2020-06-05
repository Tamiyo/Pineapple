use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::cell::RefCell;
use std::thread_local;

pub type InternIndex = usize;

thread_local!(
     static  INTERNER: RefCell<Interner> = Default::default()
);

// TODO :- Improve this
// https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
#[derive(Default)]
struct Interner {
    map: HashMap<String, usize>,
    vec: Vec<String>,
}

impl Interner {
    fn intern(&mut self, s: String) -> InternIndex {
        match self.map.entry(s.clone()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let slot = self.vec.len();
                entry.insert(slot);
                self.vec.push(s);
                slot
            }
        }
    }

    fn get(&self, i: usize) -> String {
        self.vec[i].clone()
    }
}

pub fn intern_string(s: String) -> InternIndex {
    INTERNER.with(|interner| interner.borrow_mut().intern(s))
}

pub fn get_string(i: usize) -> String {
    INTERNER.with(|interner| interner.borrow().get(i))
}
