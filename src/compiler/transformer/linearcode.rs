use crate::bytecode::string_intern::intern_string;
use crate::compiler::ir::{Expr, Oper, Stmt};
use crate::core::binop::BinOp;
use crate::core::relop::RelOp;
use crate::core::value::Type;
use crate::parser::ast;
use std::collections::{hash_map::Entry, HashMap};

type Label = usize;
type Block = Vec<crate::compiler::ir::Stmt>;

// Converts AST -> TAC
pub struct LinearCodeTransformer {
    statements: Vec<Block>,
    reg_count: usize,
    label_count: usize,
    backpatch: Vec<usize>,
}

// I'll be honest, writing this all from scratch, I feel like this code is kinda wizardy...
// or it could just be a bastardized version of recursive descent but don't @ me it works even
// though it may not be super maintainable, finicky, and gross. If I can think of a better way to
// Convert my AST to TAC then I'll change this but the literature is very annoying when trying to
// read about AST to TAC conversion for machine code.
impl LinearCodeTransformer {
    pub fn new() -> Self {
        LinearCodeTransformer {
            statements: vec![],
            reg_count: 0,
            label_count: 0,
            backpatch: vec![],
        }
    }

    fn new_temporary(&mut self) -> Oper {
        let count = self.reg_count;
        self.reg_count += 1;
        Oper::Temp(count, 0)
    }

    fn new_label(&mut self) -> usize {
        let label = self.label_count;
        self.label_count += 1;
        label
    }

    fn merge_labels(&mut self, b: usize) {
        let mut block = self.statements[b].clone();

        let mut marked: Vec<usize> = vec![0; block.len()];
        let mut label_ref: HashMap<Label, Vec<usize>> = HashMap::new();

        for (i, stmt) in self.statements[b].iter().enumerate() {
            match stmt {
                Stmt::Label(l) => {
                    let labels = match label_ref.entry(*l) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => v.insert(vec![]),
                    };
                    labels.push(i);
                }
                Stmt::Jump(j) => {
                    let labels = match label_ref.entry(*j) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => v.insert(vec![]),
                    };
                    labels.push(i);
                }
                Stmt::CJump(_, cj) => {
                    let labels = match label_ref.entry(*cj) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => v.insert(vec![]),
                    };
                    labels.push(i);
                }
                _ => (),
            };

            if i > 1 {
                if let Stmt::Label(main) = block[i - 1] {
                    if let Stmt::Label(other) = stmt {
                        marked[i] = 1;
                        for stmt in block.iter_mut().take(i + 1) {
                            if stmt.contains_label(*other) {
                                stmt.replace_label(main);
                            }
                        }
                    }
                }
            }
        }

        let filtered = block
            .iter()
            .enumerate()
            .filter(|(i, _)| marked[*i] == 0)
            .map(|(_, e)| e.clone());
        self.statements[b] = filtered.collect::<Vec<_>>();
    }

    pub fn translate(&mut self, ast: Vec<ast::Stmt>) -> Vec<Block> {
        let mut block = Block::new();
        for stmt in &ast {
            if let ast::Stmt::Function(function_sym, args, _, body) = stmt {
                block.push(Stmt::NamedLabel(*function_sym));
                for arg in args.iter().rev() {
                    block.push(Stmt::Tac(Oper::Var(arg.0, 0), Expr::Oper(Oper::StackPop)));
                }

                for stmt in body {
                    self.translate_statement(stmt, &mut block);
                }

                if let Stmt::Return(_) = block.last().unwrap() {
                } else {
                    block.push(Stmt::Return(None));
                }

                self.statements.push(block.clone());
                block.clear();
            } else {
                panic!("Expected function at outer scope!")
            }
        }

        let size = self.statements.len();
        for i in 0..size {
            self.merge_labels(i)
        }

        self.statements.clone()
    }

    fn translate_statement(&mut self, stmt: &ast::Stmt, block: &mut Block) {
        match stmt {
            ast::Stmt::If(ref cond, ref body, ref other) => {
                self.translate_if_statement(cond, body, other, block);
            }
            ast::Stmt::Expression(ref expr) => {
                self.translate_expression(expr, false, block);
            }
            ast::Stmt::Block(ref statements) => {
                for stmt in statements {
                    self.translate_statement(stmt, block);
                }
            }
            ast::Stmt::While(ref cond, ref body) => {
                self.translate_while_statement(cond, body, block);
            }
            ast::Stmt::Print(args) => {
                self.translate_print_statement(args, block);
            }
            ast::Stmt::Return(to_return) => self.translate_return(to_return, block),
            ast::Stmt::Function(function_sym, args, _, body) => {
                self.translate_function(function_sym, args, body);
            }
            _ => unimplemented!(),
        }
    }

    fn translate_return(&mut self, to_return: &Option<Box<ast::Expr>>, block: &mut Block) {
        match to_return {
            Some(expr) => {
                let res = self.translate_expression(expr, false, block);
                block.push(Stmt::Return(Some(res)))
            }
            _ => panic!("this should never be null"),
        }
    }

    fn translate_function(
        &mut self,
        function_sym: &usize,
        args: &[(usize, Type)],
        body: &[ast::Stmt],
    ) {
        let mut block_inner = Block::new();

        block_inner.push(Stmt::NamedLabel(*function_sym));
        for arg in args.iter().rev() {
            block_inner.push(Stmt::Tac(Oper::Var(arg.0, 0), Expr::Oper(Oper::StackPop)));
        }

        for stmt in body {
            self.translate_statement(stmt, &mut block_inner);
        }

        if let Stmt::Return(_) = block_inner.last().unwrap() {
        } else {
            block_inner.push(Stmt::Return(None));
        }

        self.statements.push(block_inner.clone());
        block_inner.clear();
    }

    fn translate_if_statement(
        &mut self,
        cond: &ast::Expr,
        body: &ast::Stmt,
        other: &Option<Box<ast::Stmt>>,
        block: &mut Block,
    ) {
        // Translate the condition
        let jump_if_label = self.new_label();
        let cond = Expr::Oper(self.translate_expression(cond, true, block));
        let cjump = Stmt::CJump(cond, jump_if_label);
        block.push(cjump);

        // Setup a new "label" for the inner block, which shouldn't have a goto
        let next_block_label = self.new_label();
        block.push(Stmt::Label(next_block_label));

        // Translate the body
        self.translate_statement(body, block);

        // If the "other" node is another if statement
        if let Some(stmt) = other {
            let jump = Stmt::Jump(jump_if_label); // This will get backpatched later
            self.backpatch.push(block.len());
            block.push(jump);

            // This is the label of the "new" node
            block.push(Stmt::Label(jump_if_label));

            // Translate the other
            self.translate_statement(stmt, block);

            // If we are done with if statements, begin backpatching
            match **stmt {
                ast::Stmt::If(_, _, _) => {}
                _ => {
                    let label = self.new_label();
                    block.push(Stmt::Jump(label));
                    block.push(Stmt::Label(label));

                    for patch in &self.backpatch {
                        block[*patch] = Stmt::Jump(label);
                    }
                    self.backpatch.clear();
                }
            }
        }
    }

    fn translate_while_statement(&mut self, cond: &ast::Expr, body: &ast::Stmt, block: &mut Block) {
        // Before setting up the block we isolate the condition
        let loop_label = self.new_label();
        block.push(Stmt::Jump(loop_label));
        block.push(Stmt::Label(loop_label));

        // Translate the condition
        let jump_if_label = self.new_label();
        let cond = Expr::Oper(self.translate_expression(cond, true, block));
        let cjump = Stmt::CJump(cond, jump_if_label);
        block.push(cjump);

        // Dummy label fo the next block
        let dummy_label = self.new_label();

        // Setup a label that we can wrap back to
        block.push(Stmt::Label(dummy_label));

        self.translate_statement(body, block);
        block.push(Stmt::Jump(loop_label));
        block.push(Stmt::Label(jump_if_label));
    }

    fn translate_print_statement(&mut self, args: &[ast::Expr], block: &mut Block) {
        let name = intern_string("print".to_string());
        for arg in args {
            let operand = self.translate_expression(arg, false, block);
            block.push(Stmt::StackPush(operand));
        }
        let call = Stmt::Call(name, args.len());
        block.push(call);
    }

    fn translate_expression(&mut self, expr: &ast::Expr, is_cond: bool, block: &mut Block) -> Oper {
        match expr {
            ast::Expr::Value(value) => Oper::Value(value.clone()),
            ast::Expr::Variable(n) => Oper::Var(*n, 0),
            ast::Expr::Assign(n, _, l) => self.translate_assign(n, l, block),
            ast::Expr::Call(n, args) => self.translate_call(n, args, block),
            ast::Expr::Binary(l, o, r) => self.translate_binary(l, o, r, block),
            ast::Expr::Logical(l, o, r) => self.translate_logical(l, o, r, block),
            ast::Expr::Grouping(e) => self.translate_expression(e, is_cond, block),
            ast::Expr::CastAs(e, t) => self.translate_cast(e, t, is_cond, block),
        }
    }

    fn translate_cast(
        &mut self,
        expr: &ast::Expr,
        t: &Type,
        is_cond: bool,
        block: &mut Block,
    ) -> Oper {
        if let ast::Expr::Variable(n) = expr {
            let temp = self.new_temporary();
            block.push(Stmt::Tac(temp, Expr::Oper(Oper::Var(*n, 0))));

            let res = self.translate_expression(expr, is_cond, block);
            block.push(Stmt::CastAs(temp, *t));
            temp
        } else if let ast::Expr::Value(v) = expr {
            let temp = self.new_temporary();
            block.push(Stmt::Tac(temp, Expr::Oper(Oper::Value(*v))));

            let res = self.translate_expression(expr, is_cond, block);
            block.push(Stmt::CastAs(temp, *t));
            temp
        } else if let ast::Expr::Call(n, e) = expr {
            let temp = self.new_temporary();
            let res = self.translate_expression(expr, is_cond, block);
            block.push(Stmt::Tac(temp, Expr::Oper(res)));
            block.push(Stmt::CastAs(temp, *t));
            temp
        } else if let ast::Expr::Grouping(e) = expr {
            self.translate_cast(e, t, is_cond, block)
        } else if let ast::Expr::Binary(left, op, right) = expr {
            let temp = self.new_temporary();
            let res = self.translate_binary(left, op, right, block);
            block.push(Stmt::Tac(temp, Expr::Oper(res)));
            block.push(Stmt::CastAs(temp, *t));
            temp
        } else if let ast::Expr::Logical(left, op, right) = expr {
            let temp = self.new_temporary();
            let res = self.translate_logical(left, op, right, block);
            block.push(Stmt::Tac(temp, Expr::Oper(res)));
            block.push(Stmt::CastAs(temp, *t));
            temp
        } else {
            panic!(format!("{:?}", expr))
        }
    }

    fn translate_call(&mut self, expr: &ast::Expr, args: &[ast::Expr], block: &mut Block) -> Oper {
        block.push(Stmt::StackPushAllReg);
        for arg in args {
            let res = self.translate_expression(arg, false, block);
            block.push(Stmt::StackPush(res));
        }

        let raw_str = self.translate_expression(expr, false, block);
        match raw_str {
            Oper::Var(sym, _) => {
                block.push(Stmt::Call(sym, args.len()));

                // let label = self.new_label();
                // block.push(Stmt::Label(label));
                block.push(Stmt::StackPopAllReg);

                let temp = self.new_temporary();
                block.push(Stmt::Tac(temp, Expr::Oper(Oper::ReturnValue)));
                temp
            }
            _ => panic!("Expected string as function name"),
        }
    }

    fn translate_assign(&mut self, n: &usize, l: &ast::Expr, block: &mut Block) -> Oper {
        let lval = Oper::Var(*n, 0);

        if let ast::Expr::Call(name, args) = l {
            let temp = self.translate_call(name, args, block);
            block.push(Stmt::Tac(lval, Expr::Oper(temp)))
        } else {
            let rval = Expr::Oper(self.translate_expression(l, false, block));

            let code = Stmt::Tac(lval, rval);
            block.push(code);
        }

        lval
    }

    fn translate_binary(
        &mut self,
        l: &ast::Expr,
        o: &BinOp,
        r: &ast::Expr,
        block: &mut Block,
    ) -> Oper {
        let lval = self.new_temporary();
        let rval = Expr::Binary(
            self.translate_expression(l, false, block),
            *o,
            self.translate_expression(r, false, block),
        );

        let code = Stmt::Tac(lval, rval);
        block.push(code);

        lval
    }

    fn translate_logical(
        &mut self,
        l: &ast::Expr,
        o: &RelOp,
        r: &ast::Expr,
        block: &mut Block,
    ) -> Oper {
        let lval = self.new_temporary();
        let rval = Expr::Logical(
            self.translate_expression(l, false, block),
            o.flip(),
            self.translate_expression(r, false, block),
        );

        let code = Stmt::Tac(lval, rval);
        block.push(code);

        lval
    }
}
