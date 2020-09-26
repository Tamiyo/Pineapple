use std::iter::Peekable;
use std::str::Chars;

use pineapple_error::ScanError;
use pineapple_ir::hir::token::{Token, TokenKind};
use pineapple_ir::Value;
use pineapple_session::intern_string;

pub struct Lexer<'a> {
    it: Peekable<Chars<'a>>,

    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(buf: &'a str) -> Self {
        Lexer {
            it: buf.chars().peekable(),
            line: 1,
            column: 0,
        }
    }

    fn consume_while<F>(&mut self, x: F) -> Result<Vec<char>, ScanError>
        where
            F: Fn(char) -> bool,
    {
        let mut chars: Vec<char> = Vec::new();
        while let Some(&ch) = self.it.peek() {
            if x(ch) {
                self.next()?;
                chars.push(ch);
            } else {
                break;
            }
        }
        Ok(chars)
    }

    fn next(&mut self) -> Result<char, ScanError> {
        match self.it.next() {
            Some(c) => {
                self.column += 1;
                Ok(c)
            }
            None => Err(ScanError::InputStreamEmpty),
        }
    }

    fn either(
        &mut self,
        expected: char,
        pass: TokenKind,
        fail: TokenKind,
    ) -> Result<TokenKind, ScanError> {
        match self.it.peek() {
            Some(symbol) => {
                if symbol == &expected {
                    self.next()?;
                    Ok(pass)
                } else {
                    Ok(fail)
                }
            }
            None => Ok(fail),
        }
    }

    fn whitespace(&mut self) -> Result<Option<TokenKind>, ScanError> {
        while let Some(ch) = self.it.peek() {
            match ch {
                '\t' | ' ' | '\r' => {
                    self.next()?;
                }
                '\n' => {
                    self.next()?;
                    self.line += 1;
                    self.column = 0;
                }
                '/' => {
                    self.next()?;
                    if *self.it.peek().unwrap() == '/' {
                        while let Some(ch) = self.it.peek() {
                            match ch {
                                '\n' => break,
                                _ => self.next()?,
                            };
                        }
                    } else {
                        return Ok(Some(TokenKind::Slash));
                    }
                }
                _ => break,
            }
        }

        Ok(None)
    }

    fn number(&mut self, x: char) -> Result<TokenKind, ScanError> {
        let start_line = self.line;
        let start_col = self.column;

        let mut result = String::new();
        result.push(x);

        let integer: String = self
            .consume_while(|c| c.is_numeric())?
            .into_iter()
            .collect();
        result.push_str(integer.as_str());

        if self.it.peek() == Some(&'.') {
            self.next()?;
            let decimal: String = self
                .consume_while(|c| c.is_numeric())?
                .into_iter()
                .collect();
            result.push('.');
            self.column += 1;
            result.push_str(decimal.as_str());

            let res_f = match result.parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(ScanError::InvalidNumeric(start_line, start_col));
                }
            };
            Ok(TokenKind::FloatLit(Value::from(res_f)))
        } else {
            let res_i = match result.parse::<u32>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(ScanError::InvalidNumeric(start_line, start_col));
                }
            };
            Ok(TokenKind::IntLit(Value::from(res_i)))
        }
    }

    fn string(&mut self, delim: char) -> Result<TokenKind, ScanError> {
        let start_line = self.line;
        let start_col = self.column;

        let result = self.consume_while(|c| c != delim)?;
        if self.next()? != delim {
            return Err(ScanError::UnterminatedString(start_line, start_col));
        }

        let str: String = result.iter().collect();
        Ok(TokenKind::StrLit(intern_string(str)))
    }

    fn keyword(&mut self, name: String) -> Result<TokenKind, ScanError> {
        let key = match name.as_str() {
            "and" => TokenKind::And,
            "as" => TokenKind::As,
            "elif" => TokenKind::Elif,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "if" => TokenKind::If,
            "in" => TokenKind::In,
            "my" => TokenKind::My,
            "none" => TokenKind::None,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "true" => TokenKind::True,
            "while" => TokenKind::While,

            // Types
            "int8" => TokenKind::I8Ty,
            "int16" => TokenKind::I16Ty,
            "int32" => TokenKind::I32Ty,
            "int64" => TokenKind::I64Ty,
            "int" => TokenKind::I32Ty,

            "uint8" => TokenKind::U8Ty,
            "uint16" => TokenKind::U16Ty,
            "uint32" => TokenKind::U32Ty,
            "uint64" => TokenKind::U64Ty,
            "uint" => TokenKind::U32Ty,

            "float32" => TokenKind::F32Ty,
            "float64" => TokenKind::F64Ty,

            "bool" => TokenKind::BoolTy,

            "string" => TokenKind::StringTy,
            _ => TokenKind::Ident(intern_string(name.clone())),
        };

        Ok(key)
    }

    fn identifier(&mut self, x: char) -> Result<TokenKind, ScanError> {
        let mut result = String::new();
        result.push(x);

        let rest: String = self
            .consume_while(|c| c.is_alphanumeric() || c == '_')?
            .into_iter()
            .collect();
        result.push_str(rest.as_str());

        self.keyword(result)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, ScanError> {
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            if let Some(symbol) = self.whitespace()? {
                tokens.push(Token::new(symbol, self.line, self.column));
                self.whitespace()?;
            };

            let ch = match self.next() {
                Ok(ch) => {
                    if ch == '\n' || ch == '\r' {
                        self.line += 1;
                        self.column = 0;
                    }
                    ch
                }
                Err(_) => break,
            };

            let result = match ch {
                '!' => self.either('=', TokenKind::NotEqual, TokenKind::Not)?,
                '=' => self.either('=', TokenKind::EqualEqual, TokenKind::Equal)?,
                '<' => self.either('=', TokenKind::LessEqual, TokenKind::Less)?,
                '>' => self.either('=', TokenKind::GreaterEqual, TokenKind::Greater)?,
                '+' => TokenKind::Plus,
                '-' => TokenKind::Minus,
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                '%' => TokenKind::Modulo,
                '^' => TokenKind::Carat,
                '(' => TokenKind::LeftParen,
                ')' => TokenKind::RightParen,
                '{' => TokenKind::LeftBrace,
                '}' => TokenKind::RightBrace,
                '[' => TokenKind::LeftSquare,
                ']' => TokenKind::RightSquare,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                ':' => TokenKind::Colon,
                ';' => TokenKind::Semicolon,
                '#' => TokenKind::Fun,
                '@' => TokenKind::Class,
                '$' => TokenKind::Var,
                x if x.is_numeric() => self.number(x)?,
                x if x.is_alphabetic() => self.identifier(x)?,
                '\'' | '"' => self.string(ch)?,
                _ => {
                    break;
                }
            };

            tokens.push(Token::new(result, self.line, self.column));
        }
        tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
        Ok(tokens)
    }
}
