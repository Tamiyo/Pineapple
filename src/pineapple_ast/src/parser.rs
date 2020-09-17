use crate::ast::{Expr, Stmt};
use crate::op::BinOp;
use crate::op::RelOp;
use pineapple_error::ParseError;
use pineapple_ir::token::{Token, TokenKind};
use pineapple_ir::value::Value;
use pineapple_ir::value::ValueTy;

#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assign,
    Cast,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Power,
    Unary,
    Index,
    Call,
}

impl From<&Token> for Precedence {
    fn from(token: &Token) -> Precedence {
        match token.kind {
            TokenKind::Colon | TokenKind::Equal => Precedence::Assign,
            TokenKind::As => Precedence::Cast,
            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::NotEqual | TokenKind::EqualEqual => Precedence::Equality,
            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Modulo => Precedence::Factor,
            TokenKind::Carat => Precedence::Power,
            TokenKind::Not => Precedence::Unary,
            TokenKind::LeftSquare => Precedence::Index,
            TokenKind::LeftParen | TokenKind::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut tokens = tokens;
        tokens.reverse();
        Parser { tokens }
    }

    fn peek(&self) -> Result<&Token, ParseError> {
        match self.tokens.last() {
            Some(token) => Ok(token),
            None => Err(ParseError::TokenStreamEmpty),
        }
    }

    fn next(&mut self) -> Result<Token, ParseError> {
        match self.tokens.pop() {
            Some(token) => Ok(token),
            None => Err(ParseError::TokenStreamEmpty),
        }
    }

    fn consume(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        let found = self.peek()?;
        if found.kind != expected {
            Err(ParseError::UnexpectedToken(found.clone(), expected))
        } else {
            self.next()?;
            Ok(())
        }
    }

    fn consume_type(&mut self) -> Result<ValueTy, ParseError> {
        let found = self.next()?;

        match found.kind {
            TokenKind::I8Ty => Ok(ValueTy::I8),
            TokenKind::I16Ty => Ok(ValueTy::I16),
            TokenKind::I32Ty => Ok(ValueTy::I32),
            TokenKind::I64Ty => Ok(ValueTy::I64),
            TokenKind::U8Ty => Ok(ValueTy::U8),
            TokenKind::U16Ty => Ok(ValueTy::U16),
            TokenKind::U32Ty => Ok(ValueTy::U32),
            TokenKind::U64Ty => Ok(ValueTy::U64),
            TokenKind::F32Ty => Ok(ValueTy::F32),
            TokenKind::F64Ty => Ok(ValueTy::F64),
            TokenKind::BoolTy => Ok(ValueTy::BOOL),
            TokenKind::StringTy => Ok(ValueTy::STR),
            _ => Err(ParseError::ExpectedVariableTy(found)),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = vec![];
        while self.peek()?.kind != TokenKind::Eof {
            statements.push(self.parse_declaration()?);
        }
        Ok(statements)
    }

    fn parse_declaration(&mut self) -> Result<Stmt, ParseError> {
        match self.peek()?.kind {
            _ => self.parse_statement(),
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek()?.kind {
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Print => self.parse_print_statement(),
            TokenKind::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::While)?;
        self.consume(TokenKind::LeftParen)?;
        let while_condition = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen)?;

        let while_block = self.parse_block_statement()?;

        Ok(Stmt::While(
            Box::new(while_condition),
            Box::new(while_block),
        ))
    }

    fn parse_block_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();
        while self.peek()?.kind != TokenKind::RightBrace && self.peek()?.kind != TokenKind::Eof {
            statements.push(self.parse_declaration()?);
        }

        self.consume(TokenKind::RightBrace)?;
        Ok(Stmt::Block(statements))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::If)?;
        self.consume(TokenKind::LeftParen)?;
        let if_condition = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen)?;

        let if_block = self.parse_block_statement()?;
        let rest = self.parse_elif_statement()?;

        Ok(Stmt::If(Box::new(if_condition), Box::new(if_block), rest))
    }

    fn parse_elif_statement(&mut self) -> Result<Option<Box<Stmt>>, ParseError> {
        if self.peek()?.kind == TokenKind::Elif {
            self.consume(TokenKind::Elif)?;
            self.consume(TokenKind::LeftParen)?;

            let elif_condition = self.parse_expression(Precedence::None)?;

            self.consume(TokenKind::RightParen)?;
            let elif_block = self.parse_block_statement()?;
            Ok(Some(Box::new(Stmt::If(
                Box::new(elif_condition),
                Box::new(elif_block),
                self.parse_elif_statement()?,
            ))))
        } else {
            self.parse_else_statement()
        }
    }

    fn parse_else_statement(&mut self) -> Result<Option<Box<Stmt>>, ParseError> {
        let else_block = if self.peek()?.kind == TokenKind::Else {
            self.consume(TokenKind::Else)?;
            Some(self.parse_block_statement()?)
        } else {
            Some(Stmt::Block(vec![]))
        };

        Ok(else_block.map(Box::new))
    }

    fn parse_print_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Print)?;
        self.consume(TokenKind::LeftParen)?;

        let expr_list = self.parse_expression_list()?;

        self.consume(TokenKind::RightParen)?;
        self.consume(TokenKind::Semicolon)?;

        Ok(Stmt::Print(expr_list))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Return)?;
        let expr = if self.peek()?.kind != TokenKind::Semicolon {
            Some(Box::new(self.parse_expression(Precedence::None)?))
        } else {
            None
        };
        self.consume(TokenKind::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut expressions = Vec::new();

        if self.peek()?.kind != TokenKind::RightParen {
            expressions.push(self.parse_expression(Precedence::None)?);
        }

        while self.peek()?.kind == TokenKind::Comma {
            self.consume(TokenKind::Comma)?;
            expressions.push(self.parse_expression(Precedence::None)?);
            // Should this be here?
            // i += 1;
        }

        Ok(expressions)
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Stmt::Expression(Box::new(expr)))
    }

    fn infix(&mut self, left: &mut Expr) -> Result<Expr, ParseError> {
        match self.peek()?.kind {
            TokenKind::Or
            | TokenKind::And
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Modulo
            | TokenKind::Carat => self.parse_binary(left),
            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::NotEqual
            | TokenKind::EqualEqual => self.parse_logical(left),
            TokenKind::Equal | TokenKind::Colon => self.parse_assign(left),
            TokenKind::As => self.parse_cast(left),
            // TokenKind::LeftParen => parse_call(left, expected_type),
            _ => Err(ParseError::UnexpectedInfixOperator(self.peek()?.clone())),
        }
    }

    fn prefix(&mut self) -> Result<Expr, ParseError> {
        match self.peek()?.kind {
            TokenKind::IntLit(_)
            | TokenKind::FloatLit(_)
            | TokenKind::Identifier(_)
            | TokenKind::StrLit(_) => self.parse_primary(),
            TokenKind::True => {
                self.next()?;
                Ok(Expr::Value(Value::from(true)))
            }
            TokenKind::False => {
                self.next()?;
                Ok(Expr::Value(Value::from(false)))
            }
            TokenKind::LeftParen => self.parse_grouping(),
            _ => Err(ParseError::UnexpectedPrefixOperator(self.peek()?.clone())),
        }
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr, ParseError> {
        let mut expr = self.prefix()?;
        while let Ok(token) = self.peek() {
            let next = Precedence::from(token);
            if precedence >= next {
                break;
            }
            expr = self.infix(&mut expr)?;
        }
        Ok(expr)
    }

    fn parse_cast(&mut self, left: &mut Expr) -> Result<Expr, ParseError> {
        self.consume(TokenKind::As)?;
        let ctype = self.consume_type()?;
        Ok(Expr::CastAs(Box::new(left.clone()), ctype))
    }

    fn parse_assign(&mut self, left: &mut Expr) -> Result<Expr, ParseError> {
        let mut expected_type = None;

        if self.peek()?.kind == TokenKind::Colon {
            self.consume(TokenKind::Colon)?;
            expected_type = Some(self.consume_type()?);
            self.consume(TokenKind::Equal)?;
        } else {
            if let Expr::Variable(_) = left {
                self.consume(TokenKind::Equal)?;
            }
        }

        let right = self.parse_expression(Precedence::None)?;
        match left {
            Expr::Variable(identifier) => {
                Ok(Expr::Assign(*identifier, expected_type, Box::new(right)))
            }
            _ => Err(ParseError::ExpectedLValue),
        }
    }

    fn parse_binary(&mut self, left: &mut Expr) -> Result<Expr, ParseError> {
        let precedence = Precedence::from(self.peek()?);

        let op = self.next()?;
        match op.kind {
            TokenKind::Or
            | TokenKind::And
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Modulo
            | TokenKind::Carat => Ok(()),
            _ => {
                return Err(ParseError::ExpectedBinaryOperator(op));
            }
        }?;

        let right = self.parse_expression(precedence)?;

        Ok(Expr::Binary(
            Box::new(left.clone()),
            BinOp::from(&op),
            Box::new(right),
        ))
    }

    fn parse_logical(&mut self, left: &mut Expr) -> Result<Expr, ParseError> {
        let precedence = Precedence::from(self.peek()?);

        let op = self.next()?;
        match op.kind {
            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::NotEqual
            | TokenKind::EqualEqual => Ok(()),
            _ => {
                return Err(ParseError::ExpectedBinaryOperator(op));
            }
        }?;

        let right = self.parse_expression(precedence)?;

        Ok(Expr::Logical(
            Box::new(left.clone()),
            RelOp::from(&op),
            Box::new(right),
        ))
    }

    fn parse_grouping(&mut self) -> Result<Expr, ParseError> {
        self.consume(TokenKind::LeftParen)?;
        let expr = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen)?;
        Ok(Expr::Grouping(Box::new(expr)))
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.next()?;
        match token.kind {
            TokenKind::IntLit(value) => Ok(Expr::Value(value)),
            TokenKind::FloatLit(value) => Ok(Expr::Value(value)),
            TokenKind::Identifier(sym) => Ok(Expr::Variable(sym)),
            TokenKind::StrLit(sym) => Ok(Expr::Value(Value::from(sym))),
            _ => Err(ParseError::ExpectedLiteral(token)),
        }
    }
}
