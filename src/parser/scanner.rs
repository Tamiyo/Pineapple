use crate::parser::error::ScanError;
use crate::{
    bytecode::distance::Distance,
    parser::tokens::{Symbol, Token},
};
use std::iter::Peekable;
use std::str::Chars;

/**
 *  [Scanner]
 *
 *  The scanner's job is the read a source program and to tokenize
 *  it, resulting in vector of tokens commonly refered to as the token
 *  stream.
 */
pub struct Scanner<'a> {
    /**
     *  [Iterator]
     *
     *  A peekable iterator of characters (the source input).
     */
    it: Peekable<Chars<'a>>,

    /**
     *  [Line]
     *
     *  The current line that we are
     */
    line: usize,

    /**
     *  [Column]
     *
     *  The current column that we are scanning.
     */
    column: usize,
}

impl<'a> Scanner<'a> {
    /**
     *  [Create a new Scanner]
     *
     *  Creates a new scanner from an input &str.
     */
    pub fn new(buf: &'a str) -> Self {
        Scanner {
            it: buf.chars().peekable(),
            line: 0,
            column: 0,
        }
    }

    /**
     *  [Consume While]
     *
     *  Consumes the source input while some condition holds.
     *
     *  Returns:
     *      Vec<char> : A vector of characters that were consumed.
     */
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
            Some(c) => Ok(c),
            None => Err(ScanError::InputStreamEmpty),
        }
    }

    /**
     *  [Either]
     *  
     *  Checks if the next char in the input stream matches an expected char.
     *
     *  Returns:
     *      Symbol : A symbol that is determined if the expected check passed or failed.
     */
    fn either(&mut self, expected: char, pass: Symbol, fail: Symbol) -> Result<Symbol, ScanError> {
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

    /**
     *  [Whitespace]
     *
     *  Consumes whitespace in the source input since we do
     *  not care to tokenize it.
     *
     *  Returns:
     *      Option<Symbol> : Used to deliminate DIVIDE from COMMENT
     */
    fn whitespace(&mut self) -> Result<Option<Symbol>, ScanError> {
        while let Some(ch) = self.it.peek() {
            match ch {
                '\n' | '\r' | '\t' | ' ' => {
                    self.next()?;
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
                        return Ok(Some(Symbol::Slash));
                    }
                }
                _ => break,
            }
        }

        Ok(None)
    }

    /**
     *  [Number]
     *
     *  Given that the first character is a numeric, produces a
     *  floating point or integer number.
     *  
     *  Returns:
     *      Symbol: A symbol that contains the number value that was just tokenized.
     */
    fn number(&mut self, x: char) -> Result<Symbol, ScanError> {
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
            result.push_str(decimal.as_str());
        }

        let res_f = match result.parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                return Err(ScanError::InvalidNumeric(start_line, start_col));
            }
        };

        Ok(Symbol::Number(Distance::from(res_f)))
    }

    /**
     *  [String]
     *
     *  Given that the first character is a alphanumeric, produces a
     *  string value.
     *  
     *  Returns:
     *      Symbol: A symbol that contains the string value that was just tokenized.
     */
    fn string(&mut self, delim: char) -> Result<Symbol, ScanError> {
        let start_line = self.line;
        let start_col = self.column;

        let result: String = self.consume_while(|c| c != delim)?.into_iter().collect();
        if self.next()? != delim {
            return Err(ScanError::UnterminatedString(start_line, start_col));
        }

        Ok(Symbol::String(result))
    }

    /**
     *  [Keyword]
     *
     *  Given a string value, determines if that string is a keyword.
     *  
     *  Returns:
     *      Symbol: A symbol that contains the keyword symbol or an identifier.
     */
    fn keyword(&mut self, name: String) -> Result<Symbol, ScanError> {
        let key = match name.as_str() {
            "and" => Symbol::And,
            "elif" => Symbol::Elif,
            "else" => Symbol::Else,
            "false" => Symbol::False,
            "for" => Symbol::For,
            "if" => Symbol::If,
            "in" => Symbol::In,
            "my" => Symbol::My,
            "none" => Symbol::None,
            "or" => Symbol::Or,
            "print" => Symbol::Print,
            "return" => Symbol::Return,
            "super" => Symbol::Super,
            "true" => Symbol::True,
            "while" => Symbol::While,
            _ => Symbol::Identifier(name),
        };

        Ok(key)
    }

    /**
     *  [Identifier]
     *
     *  Given that the first character is an alpha, tokenizes a new identifier.
     *  
     *  Returns:
     *      Symbol: A symbol that contains an identifier.
     */
    fn identifier(&mut self, x: char) -> Result<Symbol, ScanError> {
        let mut result = String::new();
        result.push(x);

        let rest: String = self
            .consume_while(|c| c.is_alphanumeric() || c == '_')?
            .into_iter()
            .collect();
        result.push_str(rest.as_str());

        self.keyword(result)
    }

    /**
     *  [Tokenize]
     *
     *  Tokenizes the user's source input.
     *
     *  Returns:
     *      Vec<Token> : A vector of tokens that is the user's tokenized source input.
     */
    pub fn tokenize(&mut self) -> Result<Vec<Token>, ScanError> {
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            if let Some(symbol) = self.whitespace()? {
                tokens.push(Token::new(symbol, self.line, self.column));
                self.whitespace()?;
            };

            let ch = match self.next() {
                Ok(ch) => {
                    if ch == '\n' {
                        self.line += 1;
                        self.column = 0;
                    } else {
                        self.column += 1;
                    }
                    ch
                }
                Err(_) => break,
            };

            let result = match ch {
                '!' => self.either('=', Symbol::NotEqual, Symbol::Not)?,
                '=' => self.either('=', Symbol::EqualEqual, Symbol::Equal)?,
                '<' => self.either('=', Symbol::LessEqual, Symbol::Less)?,
                '>' => self.either('=', Symbol::GreaterEqual, Symbol::Greater)?,
                '+' => Symbol::Plus,
                '-' => Symbol::Minus,
                '*' => Symbol::Star,
                '/' => Symbol::Slash,
                '%' => Symbol::Modulo,
                '^' => Symbol::Carat,
                '(' => Symbol::LeftParen,
                ')' => Symbol::RightParen,
                '{' => Symbol::LeftBrace,
                '}' => Symbol::RightBrace,
                '[' => Symbol::LeftSquare,
                ']' => Symbol::RightSquare,
                ',' => Symbol::Comma,
                '.' => Symbol::Dot,
                ':' => Symbol::Colon,
                ';' => Symbol::Semicolon,
                '#' => Symbol::Fun,
                '@' => Symbol::Class,
                '$' => Symbol::Var,
                x if x.is_numeric() => self.number(x)?,
                x if x.is_alphabetic() => self.identifier(x)?,
                '\'' | '"' => self.string(ch)?,
                _ => {
                    break;
                }
            };

            tokens.push(Token::new(result, self.line, self.column));
        }
        tokens.push(Token::new(Symbol::Eof, self.line, self.column));
        Ok(tokens)
    }
}
