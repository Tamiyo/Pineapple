use pineapple_ir::hir::token::Token;

mod lexer;

pub fn lex(buf: &str) -> Result<Vec<Token>, pineapple_error::ScanError> {
    let mut lexer = lexer::Lexer::new(buf);
    lexer.tokenize()
}
