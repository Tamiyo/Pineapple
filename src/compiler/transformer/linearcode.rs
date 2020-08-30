use crate::bytecode::string_intern::intern_string;
use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;
use crate::{
    bytecode::constant::Constant,
    compiler::ir::{Expr, Oper, Stmt},
    parser::ast,
};
use std::collections::{hash_map::Entry, HashMap};

type Label = usize;

// AST -> TAC
pub struct LinearCodeTransformer {
    statements: Vec<Vec<Stmt>>,
    block: Vec<Stmt>,
    reg_count: usize,
    label_count: usize,
    backpatch: Vec<usize>,
}

impl LinearCodeTransformer {
    pub fn new() -> Self {
        LinearCodeTransformer {
            statements: vec![],
            block: vec![],
            reg_count: 0,
            label_count: 1,
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

    pub fn translate(&mut self, ast: Vec<ast::Stmt>) -> Vec<Vec<Stmt>> {
        self.block.push(Stmt::Label(0));
        for stmt in &ast {
            if let ast::Stmt::Function(_, _, _) = stmt {
            } else {
                self.translate_statement(stmt);
            }
        }

        if !self.block.is_empty() {
            // if let Stmt::Jump(j) = self.block.last().unwrap() {
            //     self.block.push(Stmt::Label(*j));
            // }
        }
        let block = self.block.clone();
        self.statements.push(block);

        let size = self.statements.len();
        for i in 0..size {
            self.merge_labels(i)
        }

        self.statements.clone()
    }

    fn translate_statement(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::If(ref cond, ref block, ref other) => {
                self.translate_if_statement(cond, block, other);
            }
            ast::Stmt::Expression(ref expr) => {
                self.translate_expression(expr, false);
            }
            ast::Stmt::Block(ref statements) => {
                for stmt in statements {
                    self.translate_statement(stmt);
                }
            }
            ast::Stmt::While(ref cond, ref block) => {
                self.translate_while_statement(cond, block);
            }
            ast::Stmt::Print(args) => {
                self.translate_print_statement(args);
            }
            _ => unimplemented!(),
        }
    }

    fn translate_if_statement(
        &mut self,
        cond: &ast::Expr,
        block: &ast::Stmt,
        other: &Option<Box<ast::Stmt>>,
    ) {
        // Translate the condition
        let jump_if_label = self.new_label();
        let cond = Expr::Oper(self.translate_expression(cond, true));
        let cjump = Stmt::CJump(cond, jump_if_label);
        self.block.push(cjump);

        // Setup a new "label" for the inner block, which shouldn't have a goto
        let next_block_label = self.new_label();
        self.block.push(Stmt::Label(next_block_label));

        // Translate the body
        self.translate_statement(block);

        // If the "other" node is another if statement
        if let Some(stmt) = other {
            let jump = Stmt::Jump(jump_if_label); // This will get backpatched later
            self.backpatch.push(self.block.len());
            self.block.push(jump);

            // This is the label of the "new" node
            self.block.push(Stmt::Label(jump_if_label));

            // Translate the other
            self.translate_statement(stmt);

            // If we are done with if statements, begin backpatching
            match **stmt {
                ast::Stmt::If(_, _, _) => {}
                _ => {
                    let label = self.new_label();
                    self.block.push(Stmt::Jump(label));
                    self.block.push(Stmt::Label(label));

                    for patch in &self.backpatch {
                        self.block[*patch] = Stmt::Jump(label);
                    }
                    self.backpatch.clear();
                }
            }
        }
    }

    fn translate_while_statement(&mut self, cond: &ast::Expr, block: &ast::Stmt) {
        // Before setting up the block we isolate the condition
        let loop_label = self.new_label();
        self.block.push(Stmt::Jump(loop_label));
        self.block.push(Stmt::Label(loop_label));

        // Translate the condition
        let jump_if_label = self.new_label();
        let cond = Expr::Oper(self.translate_expression(cond, true));
        let cjump = Stmt::CJump(cond, jump_if_label);
        self.block.push(cjump);

        // Dummy label fo the next block
        let dummy_label = self.new_label();

        // Setup a label that we can wrap back to
        self.block.push(Stmt::Label(dummy_label));

        self.translate_statement(block);
        self.block.push(Stmt::Jump(loop_label));
        self.block.push(Stmt::Label(jump_if_label));
    }

    fn translate_print_statement(&mut self, args: &[ast::Expr]) {
        let name = intern_string("print".to_string());
        for arg in args {
            let operand = self.translate_expression(arg, false);
            self.block.push(Stmt::StackPush(operand));
        }
        let call = Stmt::Call(name, args.len());
        self.block.push(call);
    }

    fn translate_expression(&mut self, expr: &ast::Expr, is_cond: bool) -> Oper {
        match expr {
            ast::Expr::Number(d) => Oper::Constant(Constant::Number(*d)),
            ast::Expr::Boolean(b) => {
                if is_cond {
                    Oper::Constant(Constant::Boolean(!*b))
                } else {
                    Oper::Constant(Constant::Boolean(*b))
                }
            }
            ast::Expr::String(n) => Oper::Constant(Constant::String(*n)),
            ast::Expr::Variable(n) => Oper::Var(*n, 0),
            ast::Expr::Assign(n, l) => self.translate_assign(n, l),
            ast::Expr::Binary(l, o, r) => self.translate_binary(l, o, r),
            ast::Expr::Logical(l, o, r) => self.translate_logical(l, o, r),
            ast::Expr::Grouping(e) => self.translate_expression(e, is_cond),
        }
    }

    fn translate_assign(&mut self, n: &usize, l: &ast::Expr) -> Oper {
        let lval = Oper::Var(*n, 0);
        let rval = Expr::Oper(self.translate_expression(l, false));

        let code = Stmt::Tac(lval, rval);
        self.block.push(code);

        lval
    }

    fn translate_binary(&mut self, l: &ast::Expr, o: &BinOp, r: &ast::Expr) -> Oper {
        let lval = self.new_temporary();
        let rval = Expr::Binary(
            self.translate_expression(l, false),
            *o,
            self.translate_expression(r, false),
        );

        let code = Stmt::Tac(lval, rval);
        self.block.push(code);

        lval
    }

    fn translate_logical(&mut self, l: &ast::Expr, o: &RelOp, r: &ast::Expr) -> Oper {
        let lval = self.new_temporary();
        let rval = Expr::Logical(
            self.translate_expression(l, false),
            o.flip(),
            self.translate_expression(r, false),
        );

        let code = Stmt::Tac(lval, rval);
        self.block.push(code);

        lval
    }
}