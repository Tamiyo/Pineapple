use pineapple_ast::ast::{Expr, Stmt};
use pineapple_error::TypeError;
use pineapple_ir::value::Value;
use pineapple_ir::value::ValueTy;
use pineapple_session::get_string;

type Ident = usize;
type Type = ValueTy;
type Args = Vec<(Ident, Type)>;

pub fn typecheck(ast: &mut Vec<Stmt>) -> Result<(), TypeError> {
    pineapple_session::insert_symbol_table_context();
    for stmt in ast.iter() {
        if let Stmt::Function(name, args, return_ty, _) = stmt {
            let arg_types = args.iter().map(|ty| ty.1).collect();
            pineapple_session::insert_function_into_symbol_table(name, return_ty, arg_types);
        }
    }

    for stmt in ast.iter_mut() {
        check_stmt(stmt, None)?;
    }
    pineapple_session::pop_symbol_table_context();
    Ok(())
}

fn check_function(
    name: &Ident,
    args: &Args,
    return_ty: &Type,
    body: &mut Stmt,
) -> Result<(), TypeError> {
    let arg_types = args.iter().map(|ty| ty.1).collect();
    pineapple_session::insert_function_into_symbol_table(name, return_ty, arg_types);
    pineapple_session::insert_symbol_table_context();

    for (ident, value_ty) in args {
        pineapple_session::insert_variable_into_symbol_table(ident, value_ty);
    }

    check_stmt(body, Some(*return_ty))?;

    pineapple_session::pop_symbol_table_context();
    Ok(())
}

fn check_stmt(stmt: &mut Stmt, func_return_ty: Option<Type>) -> Result<(), TypeError> {
    match stmt {
        Stmt::Function(name, args, return_ty, body) => check_function(name, args, return_ty, body),
        Stmt::Block(stmts) => {
            for stmt in stmts {
                check_stmt(stmt, func_return_ty)?;
            }
            Ok(())
        }
        Stmt::If(cond, body, other) => {
            check_expr(cond, None)?;
            check_stmt(body, func_return_ty)?;

            if let Some(other) = other {
                check_stmt(other, func_return_ty)?;
            }
            Ok(())
        }
        Stmt::While(cond, body) => {
            check_expr(cond, None)?;
            check_stmt(body, func_return_ty)
        }
        Stmt::Expression(expr) => match check_expr(expr, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        Stmt::Print(_) => Ok(()),
        Stmt::Return(expr) => {
            if let Some(expr) = expr {
                let rtype = check_expr(expr, func_return_ty)?;
                if rtype != func_return_ty {
                    panic!("invalid return type!")
                }
            }
            Ok(())
        }
        _ => unimplemented!("{:?}", stmt),
    }
}

fn check_expr(expr: &mut Expr, expected_ty: Option<Type>) -> Result<Option<Type>, TypeError> {
    // Resolves an rval type, making sure that the expected type equals it
    fn resolve_rval_ty(ty1: Option<Type>, ty2: Option<Type>) -> Result<(), TypeError> {
        match (ty1, ty2) {
            (Some(ty1), Some(ty2)) => {
                if ty1 != ty2 {
                    return Err(TypeError::InvalidExprType(ty1, ty2));
                }
            }
            _ => return Err(TypeError::ExpectedNestedType),
        }
        Ok(())
    }

    match expr {
        Expr::Assign(lval, var_ty, rval) => {
            match var_ty {
                //  If the assign statement has a ty, that means it is "fresh", this identifier hasn't been assigned to before.
                //      In such a case we evaluate the expression with the type given and add it to the symbol table
                Some(ty) => {
                    let rval_ty = check_expr(rval, Some(*ty))?;
                    match resolve_rval_ty(Some(*ty), rval_ty) {
                        Ok(()) => (),
                        Err(e) => return Err(e),
                    }
                    pineapple_session::insert_variable_into_symbol_table(lval, ty);
                }
                //  If the assign statement does not have a ty, that means it is "old", this identifier has been assigned to before.
                //      In such a case we evaluate the expression with the stored in the symbol table
                None => {
                    let ty = match pineapple_session::get_variable_ty(lval) {
                        Some(ty) => ty,
                        None => panic!("undefined variable"),
                    };
                    let rval_ty = check_expr(rval, Some(ty))?;
                    match resolve_rval_ty(Some(ty), rval_ty) {
                        Ok(()) => (),
                        Err(e) => return Err(e),
                    }
                }
            }
            Ok(None)
        }
        Expr::Binary(left, _, right) => {
            let ty = check_expr(left, expected_ty)?;
            check_expr(right, ty)
        }
        Expr::Logical(left, _, right) => {
            let ty = check_expr(left, expected_ty)?;
            check_expr(right, ty)
        }
        Expr::Grouping(group) => check_expr(group, expected_ty),
        Expr::Variable(ident) => {
            match (pineapple_session::get_variable_ty(ident), expected_ty) {
                (Some(ty), Some(expected_ty)) => {
                    // If we have a variable in our symbol table AND it matches our expected type, we pass
                    if ty == expected_ty {
                        Ok(Some(expected_ty))
                    } else {
                        Err(TypeError::InvalidVariableType(*ident, expected_ty, ty))
                    }
                }
                (Some(_), None) => {
                    // If we have a variable w/ no type annotation, we pass as it exists
                    Ok(None)
                }
                _ => Err(TypeError::UndefinedVariable(*ident)),
            }
        }
        Expr::Value(value) => {
            if let Some(ty) = expected_ty {
                // If the value is not of the expected type we then we error
                let value_ty = value.get_ty();

                if value_ty != ty {
                    match value.try_implicit_cast(ty) {
                        Ok(()) => Ok(expected_ty),
                        Err(()) => Err(TypeError::InvalidValueType(value.clone(), ty, value_ty)),
                    }
                } else {
                    Ok(expected_ty)
                }
            } else {
                Ok(expected_ty)
            }
        }
        Expr::CastAs(expr, ty) => check_cast(expr, ty),
        Expr::Call(callee, args) => check_call(callee, args, expected_ty),
        _ => unimplemented!(),
    }
}

fn check_call(
    callee: &Expr,
    args: &mut Vec<Expr>,
    expected_ty: Option<Type>,
) -> Result<Option<Type>, TypeError> {
    if let Expr::Variable(ident) = callee {
        match (
            pineapple_session::get_function_ty(ident),
            pineapple_session::get_function_arg_tys(ident),
        ) {
            (Some(return_ty), Some(args_ty)) => {
                if let Some(expected_ty) = expected_ty {
                    if expected_ty != return_ty {
                        panic!("return type did not equal expected type")
                    }
                }

                for i in 0..args_ty.len() {
                    check_expr(&mut args[i], Some(args_ty[i]))?;
                }

                Ok(Some(return_ty))
            }
            _ => {
                panic!("expected function #1!");
            }
        }
    } else {
        panic!("expected function #2!")
    }
}

fn check_cast(expr: &mut Expr, ty: &mut ValueTy) -> Result<Option<Type>, TypeError> {
    if let Expr::Value(value) = expr {
        match value.try_cast(*ty) {
            Ok(()) => Ok(Some(*ty)),
            Err(()) => Err(TypeError::InvalidValueType(
                value.clone(),
                *ty,
                value.get_ty(),
            )),
        }
    } else if let Expr::Variable(ident) = expr {
        let var_ty = match pineapple_session::get_variable_ty(ident) {
            Some(ty) => ty,
            None => panic!("undefined variable"),
        };
        if !Value::check_can_cast(var_ty, *ty) {
            return Err(TypeError::InvalidVariableType(*ident, var_ty, *ty));
        } else {
            Ok(Some(*ty))
        }
    } else {
        check_expr(expr, Some(*ty))
    }
}
