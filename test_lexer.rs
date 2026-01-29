use crustyc::lexer::Lexer;

fn main() {
    let source = "struct S { int[10] arr; }";
    let mut lexer = Lexer::new(source).unwrap();
    
    loop {
        match lexer.next_token() {
            Ok(token) => {
                println!("{:?}", token);
                if matches!(token.kind, crustyc::lexer::TokenKind::Eof) {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                break;
            }
        }
    }
}

#[cfg(test)]
mod test_range {
    use crustyc::lexer::{Lexer, TokenKind};
    
    #[test]
    fn test_range_tokens() {
        let source = "0..5";
        let mut lexer = Lexer::new(source);
        
        let tok1 = lexer.next_token().unwrap();
        eprintln!("Token 1: {:?}", tok1);
        assert!(matches!(tok1.kind, TokenKind::IntLiteral(_)));
        
        let tok2 = lexer.next_token().unwrap();
        eprintln!("Token 2: {:?}", tok2);
        assert!(matches!(tok2.kind, TokenKind::DotDot));
        
        let tok3 = lexer.next_token().unwrap();
        eprintln!("Token 3: {:?}", tok3);
        assert!(matches!(tok3.kind, TokenKind::IntLiteral(_)));
    }
}
