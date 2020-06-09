use crate::bytecode::constant::Constant;
use crate::bytecode::string_intern::intern_string;
use crate::compiler::three_address::component::*;
use crate::parser::ast;
use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub mod component;

pub struct TacTranslator {
    statements: Vec<Vec<Stmt>>,

    block: Vec<Stmt>,

    reg_count: usize,

    backpatches: Vec<usize>,
}

impl TacTranslator {
    pub fn new() -> Self {
        TacTranslator {
            statements: vec![],

            block: vec![],

            reg_count: 0,

            backpatches: vec![],
        }
    }

    fn get_register(&mut self) -> Operand {
        let count = self.reg_count;
        self.reg_count += 1;
        Operand::Assignable(SSA {
            value: count,
            ssa: 0,
            is_var: false,
        })
    }

    fn get_label(&mut self) -> Label {
        Label::Label(self.block.len())
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
                    let labels = match label_ref.entry(j.goto) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => v.insert(vec![]),
                    };
                    labels.push(i);
                }
                Stmt::CJump(cj) => {
                    let labels = match label_ref.entry(cj.goto) {
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
                            if stmt.has_label(*other) {
                                stmt.change_label(main);
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
        for statement in &ast {
            if let ast::Stmt::Function(_, _, _) = statement {
            } else {
                self.block.push(Stmt::Label(Label::Label(0)));
            }
            self.translate_statement(statement);
            self.statements.push(self.block.clone());
            self.block.clear();
        }

        for i in 0..self.statements.len() {
            self.merge_labels(i);
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
            ast::Stmt::Block(ref stmts) => {
                for stmt in stmts {
                    self.translate_statement(stmt);
                }
            }
            ast::Stmt::While(ref cond, ref block) => {
                self.translate_while_statement(cond, block);
            }
            ast::Stmt::Print(args) => {
                self.translate_print_statement(args);
            }
            ast::Stmt::Return(value) => self.translate_return(value),
            ast::Stmt::Function(name, args, body) => self.translate_function(name, args, body),
            _ => unimplemented!(),
        }
    }

    fn translate_if_statement(
        &mut self,
        cond: &ast::Expr,
        block: &ast::Stmt,
        other: &Option<Box<ast::Stmt>>,
    ) {
        let label = self.get_label();

        let cjump = CJump {
            cond: Expr::Operand(self.translate_expression(cond, true)),
            goto: label,
        };

        self.block.push(Stmt::CJump(cjump));
        self.translate_statement(block);

        if let Some(stmt) = other {
            let jump = Jump { goto: label };

            self.backpatches.push(self.block.len());
            self.block.push(Stmt::Jump(jump));

            self.block.push(Stmt::Label(label));

            self.translate_statement(stmt);
            match **stmt {
                ast::Stmt::If(_, _, _) => {}
                _ => {
                    let label = self.get_label();
                    self.block.push(Stmt::Label(label));

                    for patch in &self.backpatches {
                        self.block[*patch] = Stmt::Jump(Jump { goto: label });
                    }
                    self.backpatches.clear();
                }
            }
        } else {
            self.block.push(Stmt::Label(label));
        }
    }

    fn translate_while_statement(&mut self, cond: &ast::Expr, block: &ast::Stmt) {
        let label = self.get_label();
        self.block.push(Stmt::Label(label));

        let cond = Expr::Operand(self.translate_expression(cond, true));
        let end_label = self.get_label();

        let cjump = CJump {
            cond: cond,
            goto: end_label,
        };

        self.block.push(Stmt::CJump(cjump));
        self.translate_statement(block);

        let jump = Jump { goto: label };
        self.block.push(Stmt::Jump(jump));

        self.block.push(Stmt::Label(end_label));
    }

    fn translate_print_statement(&mut self, args: &[ast::Expr]) {
        let name = intern_string("print".to_string());
        for arg in args {
            let operand = self.translate_expression(arg, false);
            self.block.push(Stmt::StackPush(operand));
        }
        self.block.push(Stmt::Call(name));
        if !args.is_empty() {
            for _ in 0..args.len() {
                self.block.push(Stmt::StackPop);
            }
        }
    }

    fn translate_return(&mut self, value: &Option<Box<ast::Expr>>) {
        if let Some(expr) = value {
            let retval = self.translate_expression(expr, false);
            self.block.push(Stmt::StackPush(retval));
        }
    }

    fn translate_function(&mut self, name: &usize, args: &[usize], body: &[ast::Stmt]) {
        self.block.push(Stmt::Label(Label::Named(*name)));
        for arg in args {
            let p = Stmt::Tac(Tac {
                lval: Operand::Assignable(SSA {
                    value: *arg,
                    ssa: 0,
                    is_var: true,
                }),
                rval: Expr::StackPop,
            });
            self.block.push(p);
        }

        for stmt in body {
            self.translate_statement(stmt);
        }
    }

    fn translate_expression(&mut self, expr: &ast::Expr, is_cond: bool) -> Operand {
        match expr {
            ast::Expr::Number(d) => Operand::Constant(Constant::Number(*d)),
            ast::Expr::Boolean(b) => {
                if is_cond {
                    Operand::Constant(Constant::Boolean(!*b))
                } else {
                    Operand::Constant(Constant::Boolean(*b))
                }
            }
            ast::Expr::Variable(n) => Operand::Assignable(SSA {
                value: *n,
                ssa: 0,
                is_var: true,
            }),
            ast::Expr::Assign(n, l) => self.translate_assign(n, l),
            ast::Expr::Binary(l, o, r) => self.translate_binary(l, o, r),
            ast::Expr::Logical(l, o, r) => self.translate_logical(l, o, r),
            ast::Expr::Grouping(e) => self.translate_expression(e, is_cond),
        }
    }

    fn translate_assign(&mut self, n: &usize, l: &ast::Expr) -> Operand {
        let res = Operand::Assignable(SSA {
            value: *n,
            ssa: 0,
            is_var: true,
        });

        let code = Tac {
            lval: res,
            rval: Expr::Operand(self.translate_expression(l, false)),
        };
        self.block.push(Stmt::Tac(code));

        res
    }

    fn translate_binary(&mut self, l: &ast::Expr, o: &BinOp, r: &ast::Expr) -> Operand {
        let res = self.get_register();
        let code = Tac {
            lval: res,
            rval: Expr::Binary(
                self.translate_expression(l, false),
                *o,
                self.translate_expression(r, false),
            ),
        };
        self.block.push(Stmt::Tac(code));

        res
    }

    fn translate_logical(&mut self, l: &ast::Expr, o: &RelOp, r: &ast::Expr) -> Operand {
        let res = self.get_register();
        let code = Tac {
            lval: res,
            rval: Expr::Logical(
                self.translate_expression(l, false),
                o.flip(),
                self.translate_expression(r, false),
            ),
        };
        self.block.push(Stmt::Tac(code));

        res
    }
}
