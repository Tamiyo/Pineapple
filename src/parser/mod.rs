use crate::bytecode::string_intern::intern_string;
use crate::parser::ast::{Expr, Stmt};
use crate::parser::error::ParseError;
use crate::parser::tokens::{Symbol, Token};
use binop::BinOp;
use relop::RelOp;

pub mod ast;
pub mod binop;
mod error;
pub mod relop;
pub mod scanner;
pub mod tokens;

/**
 *  [Precedence]
 *  
 *  Precedence is defined as the operator precedence used to
 *  parse expression statements.
 *
 *  This precedence is based off of that of a Pratt Parser.
 * 
 *  Personal Note: Bob Nystrom was right in his blog post... Pratt parsers are beautiful
 *  yet they are the most magical thing I've ever seen in computer science and that 
 *  includes hand rolling a gradient descent neural network and writing an OS from scratch
 *  for ARM64. Yeah, I know. It's THAT cool.
 */
#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assign,
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

/**
 *  [Precedence from Token]
 *
 *  Given a specific Token (that is, its' Symbol), assign a specific operator
 *  precedence to that Token.
 *
 *  Returns:
 *      Precedence : The precedence associated with a Token's Symbol.
 */
impl From<&Token> for Precedence {
    fn from(token: &Token) -> Precedence {
        match **token {
            Symbol::Equal => Precedence::Assign,
            Symbol::Or => Precedence::Or,
            Symbol::And => Precedence::And,
            Symbol::NotEqual | Symbol::EqualEqual => Precedence::Equality,
            Symbol::Less | Symbol::LessEqual | Symbol::Greater | Symbol::GreaterEqual => {
                Precedence::Comparison
            }
            Symbol::Plus | Symbol::Minus => Precedence::Term,
            Symbol::Star | Symbol::Slash | Symbol::Modulo => Precedence::Factor,
            Symbol::Carat => Precedence::Power,
            Symbol::Not => Precedence::Unary,
            Symbol::LeftSquare => Precedence::Index,
            Symbol::LeftParen | Symbol::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

/**
 *  [Parser]
 *
 *  The parser's job is to generate an initial AST while confirming
 *  that the tokens that it recieves represents a valid source program.
 */
pub struct Parser {
    /**
     *  A vector of tokens representing the user's tokenized source code.
     */
    tokens: Vec<Token>,
}

impl Parser {
    /**
     *  [Create a new Parser]
     *
     *  Creates a new parser from a vector of tokens.
     */
    pub fn new(mut tokens: Vec<Token>) -> Self {
        tokens.reverse();

        Parser { tokens }
    }

    /**
     *  [Parse]
     *
     *  The parser's public facing function, begins parsing
     *  a token stream.
     *
     *  Returns:
     *      Ok(Vec<Stmt>) : A vector of statements in AST form
     *      Err(e) : A error occured during parsing.
     */
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = vec![];
        while **self.peek()? != Symbol::Eof {
            statements.push(self.parse_declaration()?);
        }
        Ok(statements)
    }

    /**
     *  [Peek]
     *
     *  Peeks at the current token in the input stream.
     *
     *  Returns:
     *      Ok(&Token) : A token exists at the end of the input stream.
     *      Err(e) : The input stream is empty.
     */
    fn peek(&self) -> Result<&Token, ParseError> {
        match self.tokens.last() {
            Some(token) => Ok(token),
            None => Err(ParseError::TokenStreamEmpty),
        }
    }

    /**
     *  [Next]
     *
     *  Removes the last token from the token stream.
     *
     *  Returns:
     *      Ok(Token) : A token exists at the end of the input stream.
     *      Err(e) : The input stream is empty.
     */
    fn next(&mut self) -> Result<Token, ParseError> {
        match self.tokens.pop() {
            Some(token) => Ok(token),
            None => Err(ParseError::TokenStreamEmpty),
        }
    }

    /**
     *  [Consume]
     *
     *  Consumes the next token if it's symbol matches some expected symbol.
     *
     *  Returns:
     *      Ok(()) : The next token matched the expected symbol.
     *      Err(e) : The next token did not match the expected symbol.
     */
    fn consume(&mut self, expected: Symbol) -> Result<(), ParseError> {
        let found = self.peek()?;
        if **found != expected {
            Err(ParseError::UnexpectedToken(found.clone(), expected))
        } else {
            self.next()?;
            Ok(())
        }
    }

    /**
     *  [Declaration]
     *  
     *  Parses a declaration if it exists.
     *
     *  Returns:
     *      Ok(Stmt) : A statement AST node.
     *      Err(ParseError): An error that occured while parsing.
     */
    fn parse_declaration(&mut self) -> Result<Stmt, ParseError> {
        match **self.peek()? {
            Symbol::Fun => self.parse_function(),
            _ => self.parse_statement(),
        }
    }
    fn parse_identifier_list(&mut self) -> Result<Vec<usize>, ParseError> {
        let mut identifiers = Vec::new();
        let mut next = self.next()?;
        identifiers.push(match &*next {
            Symbol::Identifier(name) => Ok(intern_string(name.clone())),
            _ => return Err(ParseError::ExpectedIdentifier(next.clone())),
        }?);

        while **self.peek()? == Symbol::Comma {
            self.consume(Symbol::Comma)?;
            next = self.next()?;
            identifiers.push(match &*next {
                Symbol::Identifier(name) => Ok(intern_string(name.clone())),
                _ => return Err(ParseError::ExpectedIdentifier(next.clone())),
            }?);
        }
        Ok(identifiers)
    }

    fn parse_function(&mut self) -> Result<Stmt, ParseError> {
        self.consume(Symbol::Fun)?;
        let next = self.next()?;
        let name = match &*next {
            Symbol::Identifier(name) => Ok(intern_string(name.clone())),
            _ => Err(ParseError::ExpectedIdentifier(self.peek()?.clone())),
        }?;

        self.consume(Symbol::LeftParen)?;
        let params: Vec<usize> = if **self.peek()? != Symbol::RightParen {
            self.parse_identifier_list()?
        } else {
            Vec::new()
        };
        self.consume(Symbol::RightParen)?;

        self.consume(Symbol::LeftBrace)?;
        let mut statements: Vec<Stmt> = Vec::new();
        while **self.peek()? != Symbol::RightBrace && **self.peek()? != Symbol::Eof {
            statements.push(self.parse_declaration()?);
        }
        self.consume(Symbol::RightBrace)?;

        Ok(Stmt::Function(name, params, statements))
    }

    /**
     *  [Statement]
     *  
     *  Parses a statement if it exists.
     *
     *  Returns:
     *      Ok(Stmt) : A statement AST node.
     *      Err(ParseError): An error that occured while parsing.
     */
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match **self.peek()? {
            Symbol::If => self.parse_if_statement(),
            Symbol::While => self.parse_while_statement(),
            Symbol::Print => self.parse_print_statement(),
            Symbol::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(Symbol::If)?;
        self.consume(Symbol::LeftParen)?;
        let if_condition = self.parse_expression(Precedence::None)?;
        self.consume(Symbol::RightParen)?;

        let if_block = self.parse_block_statement()?;
        let rest = self.parse_elif_statement()?;

        Ok(Stmt::If(Box::new(if_condition), Box::new(if_block), rest))
    }

    fn parse_elif_statement(&mut self) -> Result<Option<Box<Stmt>>, ParseError> {
        if **self.peek()? == Symbol::Elif {
            self.consume(Symbol::Elif)?;
            self.consume(Symbol::LeftParen)?;
            let elif_condition = self.parse_expression(Precedence::None)?;
            self.consume(Symbol::RightParen)?;
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
        let else_block = if **self.peek()? == Symbol::Else {
            self.consume(Symbol::Else)?;
            Some(self.parse_block_statement()?)
        } else {
            None
        };

        Ok(else_block.map(Box::new))
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(Symbol::While)?;
        self.consume(Symbol::LeftParen)?;
        let while_condition = self.parse_expression(Precedence::None)?;
        self.consume(Symbol::RightParen)?;

        let while_block = self.parse_block_statement()?;

        Ok(Stmt::While(
            Box::new(while_condition),
            Box::new(while_block),
        ))
    }

    fn parse_print_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(Symbol::Print)?;
        self.consume(Symbol::LeftParen)?;
        let expr_list = self.parse_expression_list()?;
        self.consume(Symbol::RightParen)?;
        self.consume(Symbol::Semicolon)?;

        Ok(Stmt::Print(expr_list))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(Symbol::Return)?;
        let expr = if **self.peek()? != Symbol::Semicolon {
            Some(Box::new(self.parse_expression(Precedence::None)?))
        } else {
            None
        };
        self.consume(Symbol::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_block_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(Symbol::LeftBrace)?;

        let mut statements = Vec::new();
        while **self.peek()? != Symbol::RightBrace && **self.peek()? != Symbol::Eof {
            statements.push(self.parse_declaration()?);
        }

        self.consume(Symbol::RightBrace)?;
        Ok(Stmt::Block(statements))
    }

    /**
     *  [Expression Statement]
     *  
     *  Parses a expression statement.
     *
     *  Returns:
     *      Ok(Stmt) : A expression_statement AST node.
     *      Err(ParseError): An error that occured while parsing.
     */
    fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression(Precedence::None)?;
        self.consume(Symbol::Semicolon)?;
        Ok(Stmt::Expression(Box::new(expr)))
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut expressions = Vec::new();
        if **self.peek()? != Symbol::RightParen {
            expressions.push(self.parse_expression(Precedence::None)?);
        }

        while **self.peek()? == Symbol::Comma {
            self.consume(Symbol::Comma)?;
            expressions.push(self.parse_expression(Precedence::None)?);
        }
        Ok(expressions)
    }

    /**
     *  [Expression]
     *  
     *  Parses a expression. Implemented after a Pratt Parser.
     *
     *  Returns:
     *      Ok(Expr) : A expression AST node.
     *      Err(ParseError): An error that occured while parsing.
     */
    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr, ParseError> {
        fn infix(parser: &mut Parser, left: Expr) -> Result<Expr, ParseError> {
            match **parser.peek()? {
                Symbol::Or
                | Symbol::And
                | Symbol::Plus
                | Symbol::Minus
                | Symbol::Star
                | Symbol::Slash
                | Symbol::Modulo
                | Symbol::Carat => parser.parse_binary(left),
                Symbol::Less
                | Symbol::LessEqual
                | Symbol::Greater
                | Symbol::GreaterEqual
                | Symbol::NotEqual
                | Symbol::EqualEqual => parser.parse_logical(left),
                Symbol::Equal => parser.parse_assign(left),
                // Symbol::LeftParen => parser.parse_call(left),
                _ => Err(ParseError::UnexpectedInfixOperator(parser.peek()?.clone())),
            }
        }

        fn prefix(parser: &mut Parser) -> Result<Expr, ParseError> {
            match **parser.peek()? {
                Symbol::Number(_) 
                | Symbol::Identifier(_)
                | Symbol::String(_) => parser.parse_primary(),
                Symbol::True => {
                    parser.next()?;
                    Ok(Expr::Boolean(true))
                }
                Symbol::False => {
                    parser.next()?;
                    Ok(Expr::Boolean(false))
                }
                Symbol::LeftParen => parser.parse_grouping(),
                _ => Err(ParseError::UnexpectedPrefixOperator(parser.peek()?.clone())),
            }
        }

        let mut expr = prefix(self)?;
        while let Ok(token) = self.peek() {
            let next = Precedence::from(token);
            if precedence >= next {
                break;
            }
            expr = infix(self, expr)?;
        }
        Ok(expr)
    }

    /**
     *  [Binary Expression]
     *  
     *  Parses a binary expression, consisting of
     *  a left side, a operator, and a right side.
     *
     *  Returns:
     *      Ok(Expr) : A binary_expression AST node.
     *      Err(ParseError): An error that occured while parsing.
     */
    fn parse_binary(&mut self, left: Expr) -> Result<Expr, ParseError> {
        let precedence = Precedence::from(self.peek()?);

        let op = self.next()?;
        match *op {
            Symbol::Or
            | Symbol::And
            | Symbol::Plus
            | Symbol::Minus
            | Symbol::Star
            | Symbol::Slash
            | Symbol::Modulo
            | Symbol::Carat => Ok(()),
            _ => {
                return Err(ParseError::ExpectedBinaryOperator(op));
            }
        }?;

        let right = self.parse_expression(precedence)?;

        Ok(Expr::Binary(
            Box::new(left),
            BinOp::from(&op),
            Box::new(right),
        ))
    }

    fn parse_logical(&mut self, left: Expr) -> Result<Expr, ParseError> {
        let precedence = Precedence::from(self.peek()?);

        let op = self.next()?;
        match *op {
            Symbol::Less
            | Symbol::LessEqual
            | Symbol::Greater
            | Symbol::GreaterEqual
            | Symbol::NotEqual
            | Symbol::EqualEqual => Ok(()),
            _ => {
                return Err(ParseError::ExpectedBinaryOperator(op));
            }
        }?;

        let right = self.parse_expression(precedence)?;

        Ok(Expr::Logical(
            Box::new(left),
            RelOp::from(&op),
            Box::new(right),
        ))
    }

    fn parse_assign(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.consume(Symbol::Equal)?;
        let right = self.parse_expression(Precedence::None)?;
        match left {
            Expr::Variable(identifier) => Ok(Expr::Assign(identifier, Box::new(right))),
            _ => Err(ParseError::ExpectedLValue(left)),
        }
    }

    // fn parse_call(&mut self, left: Expr) -> Result<Expr, ParseError> {
    //     self.consume(Symbol::LeftParen)?;
    //     let args = self.parse_expression_list()?;
    //     self.consume(Symbol::RightParen)?;
    //     Ok(Expr::Call(Box::new(left), args))
    // }

    /**
     *  [Primary Expression]
     *  
     *  Parses a primary expression, consisting of some literal.
     *
     *  Returns:
     *      Ok(Expr) : A binary_expression AST node.
     *      Err(ParseError): An error that occured while parsing.
     */
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.next()?;
        match token.sym {
            Symbol::Number(number) => Ok(Expr::Number(number)),
            Symbol::Identifier(identifier) => {
                let intern = intern_string(identifier);
                Ok(Expr::Variable(intern))
            }
            Symbol::String(str) => {
                let intern = intern_string(str);
                Ok(Expr::String(intern))
            }
            _ => Err(ParseError::ExpectedLiteral(token)),
        }
    }

    /**
     *  [Primary Grouping]
     *  
     *  Parses a grouping expression, consisting of some expression
     *  surrounded by parenthesis
     *  
     *
     *  Returns:
     *      Ok(Expr) : A grouping_expression AST node.
     *      Err(ParseError): An error that occured while parsing.
     */
    fn parse_grouping(&mut self) -> Result<Expr, ParseError> {
        self.consume(Symbol::LeftParen)?;
        let expr = self.parse_expression(Precedence::None)?;
        self.consume(Symbol::RightParen)?;
        Ok(Expr::Grouping(Box::new(expr)))
    }
}
