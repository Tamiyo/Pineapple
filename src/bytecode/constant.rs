use crate::bytecode::distance::Distance;
use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;

use std::fmt;
use std::hash::Hash;

/**
 *  [Constant]
 *
 *  This constant representation is what the compiler will
 *  use to represent constant values that it picks up during
 *  it's compilation phases.
 *
 *  Unlike it's runtime counterpart, compiletime constants
 *  disregard many runtime attributes and can be seen as
 *  a more barebones implmenetation.
 */
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Constant {
    Number(Distance),
    Identifier(usize),
    Boolean(bool),
    None,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Constant::Number(float) => write!(f, "{}", Into::<f64>::into(float)),
            Constant::Identifier(identifier) => write!(f, "{}", *identifier),
            Constant::Boolean(b) => {
                if *b {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Constant::None => write!(f, "None"),
        }
    }
}

impl Constant {
    pub fn compute_binary(&self, op: BinOp, other: Constant) -> Constant {
        match op {
            BinOp::Or => match (self, other) {
                (Constant::Boolean(b1), Constant::Boolean(b2)) => Constant::Boolean(*b1 || b2),
                _ => panic!(""),
            },
            BinOp::And => match (self, other) {
                (Constant::Boolean(b1), Constant::Boolean(b2)) => Constant::Boolean(*b1 && b2),
                _ => panic!(""),
            },
            BinOp::Plus => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Number(Distance::from(f1 + f2))
                }
                _ => panic!(""),
            },
            BinOp::Minus => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Number(Distance::from(f1 - f2))
                }
                _ => panic!(""),
            },
            BinOp::Slash => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Number(Distance::from(f1 / f2))
                }
                _ => panic!(""),
            },
            BinOp::Modulo => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Number(Distance::from(f1 % f2))
                }
                _ => panic!(""),
            },
            BinOp::Carat => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Number(Distance::from(f64::powf(f1, f2)))
                }
                _ => panic!(""),
            },
            BinOp::Star => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Number(Distance::from(f1 * f2))
                }
                _ => panic!(""),
            },
            _ => unimplemented!(),
        }
    }

    pub fn compute_logical(&self, op: RelOp, other: Constant) -> Constant {
        match op {
            RelOp::NotEqual => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Boolean((f1 - f2).abs() > f64::EPSILON)
                }
                (Constant::Boolean(b1), Constant::Boolean(b2)) => Constant::Boolean(*b1 != b2),
                _ => panic!(""),
            },
            RelOp::EqualEqual => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Boolean((f1 - f2).abs() < f64::EPSILON)
                }
                (Constant::Boolean(b1), Constant::Boolean(b2)) => Constant::Boolean(*b1 == b2),
                _ => panic!(""),
            },
            RelOp::Greater => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Boolean(f1 > f2)
                }
                _ => panic!(""),
            },
            RelOp::GreaterEqual => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Boolean(f1 >= f2)
                }
                (a, b) => panic!(format!("expected two numbers, got {:?} and {:?} instead", a, b)),
            },
            RelOp::Less => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Boolean(f1 < f2)
                }
                _ => panic!(""),
            },
            RelOp::LessEqual => match (self, other) {
                (Constant::Number(n1), Constant::Number(n2)) => {
                    let f1 = Into::<f64>::into(n1);
                    let f2 = Into::<f64>::into(n2);
                    Constant::Boolean(f1 <= f2)
                }
                _ => panic!(""),
            },
        }
    }
}
