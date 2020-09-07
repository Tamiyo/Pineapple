use super::error::BindingError;
use crate::bytecode::chunk::Chunk;
use crate::bytecode::module::Module;
use crate::parser::ast::{Expr, Stmt};
use crate::{
    bytecode::string_intern::intern_string, core::value::Type, semantic_analysis::local::*,
};

type Sym = usize;

#[derive(Debug, PartialEq)]
pub enum ContextType {
    Function,
    Method,
    Initializer,
    Script,
}

#[derive(Debug)]
pub struct BinderContext {
    pub context_type: ContextType,
    pub enclosing: usize,
    pub no_enclosing: bool,
    pub chunk_index: usize,
    pub locals: Locals,
    pub upvalues: Vec<Upvalue>,
}

impl BinderContext {
    pub fn new(
        context_type: ContextType,
        enclosing: usize,
        no_enclosing: bool,
        chunk_index: usize,
    ) -> Self {
        let mut locals = Locals::new();

        match context_type {
            ContextType::Function => {
                let sym = intern_string("".to_string());
                locals.insert(sym);
            }
            _ => {
                let sym = intern_string("my".to_string());
                locals.insert(sym);
                locals.mark_initialized();
            }
        };

        BinderContext {
            context_type,
            enclosing,
            no_enclosing,
            chunk_index,
            locals,
            upvalues: vec![],
        }
    }

    fn add_upvalue(&mut self, slot: usize, is_local: bool) -> usize {
        for (index, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.slot == slot && upvalue.is_local == is_local {
                return index;
            }
        }
        self.upvalues.push(Upvalue { slot, is_local });

        self.upvalues.len() - 1
    }

    pub fn resolve_local(&self, sym: Sym) -> Result<Option<usize>, BindingError> {
        if let Some(local) = self.locals.get(sym) {
            if local.is_initialized {
                Ok(Some(local.slot))
            } else {
                Err(BindingError::PlaceHolder)
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct ClassContext {
    pub enclosing: usize,
    pub superclass: bool,
    pub name: Sym,
}

#[derive(Debug)]
pub struct Binder {
    module: Module,
    contexts: Vec<BinderContext>,
    classes: Vec<ClassContext>,
}

impl Binder {
    pub fn new() -> Self {
        let chunk_index = 0;

        let mut contexts: Vec<BinderContext> = vec![];
        contexts.push(BinderContext::new(
            ContextType::Script,
            0,
            true,
            chunk_index,
        ));

        let mut module = Module::new();
        module.add_chunk();

        Binder {
            module,
            contexts,
            classes: vec![],
        }
    }

    fn current_class(&self) -> Option<&ClassContext> {
        self.classes.last()
    }

    fn current_context(&self) -> &BinderContext {
        self.contexts
            .last()
            .expect("expected a &BinderContext to exist")
    }

    fn current_context_mut(&mut self) -> &mut BinderContext {
        self.contexts
            .last_mut()
            .expect("expected a &mut BinderContext to exist")
    }

    fn current_chunk(&self) -> &Chunk {
        self.module.get_chunk(self.current_context().chunk_index)
    }

    fn current_chunk_mut(&mut self) -> &mut Chunk {
        self.module
            .get_chunk_mut(self.current_context().chunk_index)
    }

    fn resolve_local(&mut self, sym: Sym) -> Result<Option<usize>, BindingError> {
        self.current_context_mut().resolve_local(sym)
    }

    fn resolve_upvalue(
        &mut self,
        context_index: usize,
        sym: Sym,
    ) -> Result<Option<usize>, BindingError> {
        let context = &self.contexts[context_index];
        let enclosing = context.enclosing;
        if !context.no_enclosing {
            if let Some(slot) = self.contexts[enclosing].resolve_local(sym)? {
                let enclosing_mut = &mut self.contexts[enclosing];
                enclosing_mut.locals.mark_captured(slot);
                let index = self.contexts[context_index].add_upvalue(slot, true);
                return Ok(Some(index));
            } else if let Some(slot) = self.resolve_upvalue(enclosing, sym)? {
                let index = self.contexts[context_index].add_upvalue(slot, false);
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    fn begin_scope(&mut self) {
        self.current_context_mut().locals.enter_scope();
    }

    fn end_scope(&mut self) {
        let locals = self.current_context_mut().locals.leave_scope();
    }

    pub fn compile(&mut self, statements: &[Stmt]) -> Result<(), BindingError> {
        self.compile_program(statements)?;
        Ok(())
    }

    fn compile_program(&mut self, statements: &[Stmt]) -> Result<(), BindingError> {
        for statement in statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, statement: &Stmt) -> Result<(), BindingError> {
        match statement {
            Stmt::Expression(ref expr) => self.compile_expression_statement(expr),
            Stmt::Print(ref expr_list) => self.compile_print(expr_list),
            Stmt::Return(ref expr) => self.compile_return(expr),
            Stmt::Block(ref statements) => self.compile_block(statements),
            Stmt::If(ref condition, ref body, ref next) => self.compile_if(condition, body, next),
            Stmt::While(ref condition, ref body) => self.compile_while(condition, body),
            Stmt::Function(ref sym, ref params, _, ref body) => {
                self.compile_function(*sym, params, body)
            }
            _ => unimplemented!(),
        }?;

        Ok(())
    }

    fn compile_expression_statement(&mut self, expr: &Expr) -> Result<(), BindingError> {
        match expr {
            Expr::Assign(_, _, _) => {
                self.compile_expression(expr)?;
            }
            _ => {
                self.compile_expression(expr)?;
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expr) -> Result<(), BindingError> {
        match expr {
            Expr::Value(_) => Ok(()),
            Expr::Variable(ref sym) => self.compile_variable(*sym),
            Expr::Assign(ref sym, _, ref right) => self.compile_assign(*sym, right),
            Expr::Binary(ref left, _, ref right) => self.compile_binary(left, right),
            Expr::Logical(ref left, _, ref right) => self.compile_logical(left, right),
            Expr::Grouping(ref expr) => self.compile_grouping(expr),
            Expr::Call(ref callee, ref arguments) => self.compile_call(callee, arguments),
            _ => unimplemented!("{:?}", expr),
        }?;
        Ok(())
    }

    fn compile_variable(&mut self, sym: Sym) -> Result<(), BindingError> {
        if let Some(local) = self.resolve_local(sym)? {
        } else if let Some(upvalue) = self.resolve_upvalue(self.contexts.len() - 1, sym)? {
        } else {
        }
        Ok(())
    }

    fn compile_list(&mut self, expressions: &[Expr]) -> Result<(), BindingError> {
        for expression in expressions {
            self.compile_expression(expression)?;
        }

        Ok(())
    }

    fn compile_binary(&mut self, left: &Expr, right: &Expr) -> Result<(), BindingError> {
        self.compile_expression(left)?;
        self.compile_expression(right)?;

        Ok(())
    }

    fn compile_logical(&mut self, left: &Expr, right: &Expr) -> Result<(), BindingError> {
        self.compile_expression(left)?;
        self.compile_expression(right)?;

        Ok(())
    }

    fn compile_and(&mut self, left: &Expr, right: &Expr) -> Result<(), BindingError> {
        self.compile_expression(left)?;
        self.compile_expression(right)?;
        Ok(())
    }

    fn compile_or(&mut self, left: &Expr, right: &Expr) -> Result<(), BindingError> {
        self.compile_expression(left)?;
        self.compile_expression(right)?;
        Ok(())
    }

    fn compile_grouping(&mut self, expr: &Expr) -> Result<(), BindingError> {
        self.compile_expression(expr)?;
        Ok(())
    }
    fn compile_call(&mut self, callee: &Expr, arguments: &[Expr]) -> Result<(), BindingError> {
        self.compile_expression(callee)?;
        for arg in arguments {
            self.compile_expression(arg)?;
        }
        Ok(())
    }

    fn compile_print(&mut self, expr_list: &[Expr]) -> Result<(), BindingError> {
        for expr in expr_list {
            self.compile_expression(expr)?;
        }
        Ok(())
    }

    fn compile_return(&mut self, expr: &Option<Box<Expr>>) -> Result<(), BindingError> {
        if self.current_context().context_type == ContextType::Script {
            Err(BindingError::PlaceHolder)
        } else if self.current_context().context_type == ContextType::Initializer {
            Err(BindingError::PlaceHolder)
        } else {
            if let Some(expr) = expr {
                self.compile_expression(expr)?;
            } else {
            }
            Ok(())
        }
    }

    fn declare_variable(&mut self, sym: Sym) {
        if self.current_context().locals.depth > 0 {
            self.current_context_mut().locals.insert(sym);
        }
    }

    fn define_variable(&mut self, sym: Sym) {
        if self.current_context().locals.depth > 0 {
            self.current_context_mut().locals.mark_initialized();
        } else {
        }
    }

    fn compile_assign(&mut self, sym: Sym, expr: &Expr) -> Result<(), BindingError> {
        self.declare_variable(sym);
        self.compile_expression(expr)?;
        self.define_variable(sym);

        if let Some(local) = self.resolve_local(sym)? {
        } else if let Some(upvalue) = self.resolve_upvalue(self.contexts.len() - 1, sym)? {
        }
        Ok(())
    }

    fn compile_block(&mut self, statements: &[Stmt]) -> Result<(), BindingError> {
        for statement in statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn compile_if(
        &mut self,
        condition: &Expr,
        body: &Stmt,
        next: &Option<Box<Stmt>>,
    ) -> Result<(), BindingError> {
        self.compile_expression(condition)?;
        self.compile_statement(body)?;

        if let Some(statement) = next {
            self.compile_statement(statement)?;
        } else {
        }

        Ok(())
    }

    fn compile_while(&mut self, condition: &Expr, body: &Stmt) -> Result<(), BindingError> {
        let start_jump = self.current_chunk().instructions.len();
        self.compile_expression(condition)?;
        self.compile_statement(body)?;

        Ok(())
    }

    fn compile_function(
        &mut self,
        sym: Sym,
        params: &[(Sym, Type)],
        body: &[Stmt],
    ) -> Result<(), BindingError> {
        self.declare_variable(sym);
        if self.current_context().locals.depth > 0 {
            self.current_context_mut().locals.mark_initialized();
        }

        let chunk_index = self.module.add_chunk();
        let enclosing = self.contexts.len() - 1;
        self.contexts.push(BinderContext::new(
            ContextType::Function,
            enclosing,
            false,
            chunk_index,
        ));

        self.begin_scope();

        for param in params {
            self.declare_variable(param.0);
            self.define_variable(param.0);
        }

        self.compile_block(body)?;

        match body.last() {
            Some(Stmt::Return(_)) => (),
            _ => {
                if self.current_context().context_type == ContextType::Initializer {
                } else {
                }
            }
        };

        let context = self
            .contexts
            .pop()
            .expect("Expect a context during function compilation");

        self.define_variable(sym);
        Ok(())
    }
}
