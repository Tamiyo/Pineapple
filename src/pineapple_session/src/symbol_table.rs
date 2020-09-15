use std::cell::RefCell;
use std::collections::HashMap;

use pineapple_ir::value::ValueTy;

type Ident = usize;

thread_local! {
    static SYMBOL_TABLE: RefCell<SymbolTable> = Default::default()
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum SymbolTy {
    Variable,
    Function,
}
#[derive(Default)]
struct SymbolTable {
    stack: Vec<SymbolTableContext>,
}

impl SymbolTable {
    pub fn add_table(&mut self) {
        self.stack.push(SymbolTableContext::default());
    }

    pub fn get_current_context(&self) -> &SymbolTableContext {
        match self.stack.last() {
            Some(context) => context,
            None => panic!("unchecked: expected symbol table context"),
        }
    }

    pub fn get_current_context_mut(&mut self) -> &mut SymbolTableContext {
        match self.stack.last_mut() {
            Some(context) => context,
            None => panic!("unchecked: expected symbol table context"),
        }
    }

    pub fn pop_table(&mut self) {
        self.stack.pop();
    }
}

#[derive(Debug, Default)]
struct SymbolTableContext {
    table: HashMap<Ident, SymbolData>,
}

impl SymbolTableContext {
    pub fn new() -> Self {
        SymbolTableContext {
            table: HashMap::new(),
        }
    }

    pub fn insert_variable(&mut self, key: Ident, assoc_ty: ValueTy) {
        self.table.insert(
            key,
            SymbolData {
                symbol_ty: SymbolTy::Variable,
                assoc_ty,
            },
        );
    }

    pub fn insert_function(&mut self, key: Ident, assoc_ty: ValueTy) {
        self.table.insert(
            key,
            SymbolData {
                symbol_ty: SymbolTy::Function,
                assoc_ty,
            },
        );
    }

    pub fn contains_variable(&self, key: &Ident) -> bool {
        match self.table.get(key) {
            Some(data) => data.symbol_ty == SymbolTy::Variable,
            None => false,
        }
    }

    pub fn contains_function(&self, key: &Ident) -> bool {
        match self.table.get(key) {
            Some(data) => data.symbol_ty == SymbolTy::Function,
            None => false,
        }
    }

    pub fn get_variable_ty(&self, key: &Ident) -> Option<ValueTy> {
        match self.table.get(key) {
            Some(data) => {
                if data.symbol_ty == SymbolTy::Variable {
                    Some(data.assoc_ty)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn get_function(&self, key: &Ident) -> Option<ValueTy> {
        match self.table.get(key) {
            Some(data) => {
                if data.symbol_ty == SymbolTy::Function {
                    Some(data.assoc_ty)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct SymbolData {
    // Class, Fn, Variable, etc
    pub symbol_ty: SymbolTy,

    // The type associated with the symbol. This can be a return type, a primitive type, etc
    pub assoc_ty: ValueTy,
}

pub fn insert_context() {
    SYMBOL_TABLE.with(|table| table.borrow_mut().add_table());
}

pub fn pop_context() {
    SYMBOL_TABLE.with(|table| table.borrow_mut().pop_table());
}

pub fn insert_variable(ident: Ident, assoc_ty: ValueTy) {
    SYMBOL_TABLE.with(|table| {
        table
            .borrow_mut()
            .get_current_context_mut()
            .insert_variable(ident, assoc_ty)
    });
}

pub fn insert_function(ident: Ident, assoc_ty: ValueTy) {
    SYMBOL_TABLE.with(|table| {
        table
            .borrow_mut()
            .get_current_context_mut()
            .insert_function(ident, assoc_ty)
    });
}

pub fn get_variable_ty(ident: &Ident) -> Option<ValueTy> {
    SYMBOL_TABLE.with(|table| table.borrow().get_current_context().get_variable_ty(ident))
}

pub fn get_function_ty(ident: &Ident) -> Option<ValueTy> {
    SYMBOL_TABLE.with(|table| table.borrow().get_current_context().get_function(ident))
}
