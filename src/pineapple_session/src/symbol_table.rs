use pineapple_ir::value::ValueTy;
use std::cell::RefCell;
use std::collections::HashMap;

type Ident = usize;

thread_local! {
    static SYMBOL_TABLE: RefCell<SymbolTable> = Default::default()
}
#[derive(Default)]
struct SymbolTable {
    stack: Vec<SymbolTableContext>,
}

impl SymbolTable {
    pub fn add_table(&mut self) {
        self.stack.push(SymbolTableContext::default());
    }

    pub fn get_current_context_mut(&mut self) -> &mut SymbolTableContext {
        match self.stack.last_mut() {
            Some(context) => context,
            None => panic!("unchecked: expected symbol table context"),
        }
    }

    pub fn search_table_for_variable_ty(&self, ident: &Ident) -> Option<ValueTy> {
        let mut index = self.stack.len() - 1;

        loop {
            if let Some(ty) = self.stack[index].get_variable_ty(ident) {
                return Some(ty);
            }

            if index == 0 {
                return None;
            }
            index -= 1;
        }
    }

    pub fn search_table_for_function_ty(&self, ident: &Ident) -> Option<ValueTy> {
        let mut index = self.stack.len() - 1;

        loop {
            if let Some(ty) = self.stack[index].get_function_ty(ident) {
                return Some(ty);
            }

            if index == 0 {
                return None;
            }
            index -= 1;
        }
    }

    pub fn search_table_for_function_arg_tys(&self, ident: &Ident) -> Option<Vec<ValueTy>> {
        let mut index = self.stack.len() - 1;

        loop {
            if let Some(arg_tys) = self.stack[index].get_function_arg_tys(ident) {
                return Some(arg_tys);
            }

            if index == 0 {
                return None;
            }
            index -= 1;
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
    pub fn insert_variable(&mut self, ident: Ident, var_type: ValueTy) {
        self.table
            .insert(ident, SymbolData::Var(VariableSymbolData { var_type }));
    }

    pub fn insert_function(&mut self, ident: Ident, return_type: ValueTy, arg_types: Vec<ValueTy>) {
        self.table.insert(
            ident,
            SymbolData::Fun(FunctionSymbolData {
                return_type,
                arg_types,
            }),
        );
    }

    pub fn get_variable_ty(&self, ident: &Ident) -> Option<ValueTy> {
        match self.table.get(ident) {
            Some(SymbolData::Var(data)) => Some(data.var_type),
            _ => None,
        }
    }

    pub fn get_function_ty(&self, ident: &Ident) -> Option<ValueTy> {
        match self.table.get(ident) {
            Some(SymbolData::Fun(data)) => Some(data.return_type),
            _ => None,
        }
    }

    pub fn get_function_arg_tys(&self, ident: &Ident) -> Option<Vec<ValueTy>> {
        match self.table.get(ident) {
            Some(SymbolData::Fun(data)) => Some(data.arg_types.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct FunctionSymbolData {
    return_type: ValueTy,
    arg_types: Vec<ValueTy>,
}

#[derive(Debug, Copy, Clone)]
struct VariableSymbolData {
    var_type: ValueTy,
}

#[derive(Debug, Clone)]
enum SymbolData {
    Var(VariableSymbolData),
    Fun(FunctionSymbolData),
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

pub fn insert_function(ident: Ident, return_ty: ValueTy, arg_types: Vec<ValueTy>) {
    SYMBOL_TABLE.with(|table| {
        table
            .borrow_mut()
            .get_current_context_mut()
            .insert_function(ident, return_ty, arg_types)
    });
}

pub fn get_variable_ty(ident: &Ident) -> Option<ValueTy> {
    SYMBOL_TABLE.with(|table| table.borrow().search_table_for_variable_ty(ident))
}

pub fn get_function_ty(ident: &Ident) -> Option<ValueTy> {
    SYMBOL_TABLE.with(|table| table.borrow().search_table_for_function_ty(ident))
}

pub fn get_function_arg_tys(ident: &Ident) -> Option<Vec<ValueTy>> {
    SYMBOL_TABLE.with(|table| table.borrow().search_table_for_function_arg_tys(ident))
}
