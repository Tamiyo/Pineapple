use pineapple_ir::ValueTy;

mod string_interner;
mod symbol_table;

type Ident = usize;

pub fn intern_string(s: String) -> usize {
    string_interner::intern_string(s)
}

pub fn get_string(idx: usize) -> String {
    string_interner::get_string(idx)
}

pub fn insert_symbol_table_context() {
    symbol_table::insert_context();
}

pub fn pop_symbol_table_context() {
    symbol_table::pop_context();
}

pub fn insert_variable_into_symbol_table(ident: &Ident, value_ty: &ValueTy) {
    symbol_table::insert_variable(*ident, *value_ty);
}

pub fn insert_function_into_symbol_table(
    ident: &Ident,
    value_ty: &ValueTy,
    arg_types: Vec<ValueTy>,
) {
    symbol_table::insert_function(*ident, *value_ty, arg_types);
}

pub fn get_variable_ty(ident: &Ident) -> Option<ValueTy> {
    symbol_table::get_variable_ty(ident)
}

pub fn get_function_ty(ident: &Ident) -> Option<ValueTy> {
    symbol_table::get_function_ty(ident)
}

pub fn get_function_arg_tys(ident: &Ident) -> Option<Vec<ValueTy>> {
    symbol_table::get_function_arg_tys(ident)
}
