mod token;
mod lexer;
mod ast;
mod parser;

// use crate::lexer::*;
use crate::parser::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(args.len() > 0);

    if args.len() != 2 {
        println!("usage: {} file", args[0]);
        return;
    }


    /*
    let source = std::fs::read_to_string(&args[1]).expect("Unable to open file!");
    let mut lexer = Lexer::new(source);
    loop {
        let token = lexer.next_token();

        println!("{:?}", token);

        match token.kind {
            TokenKind::Error(message) => panic!(message),
            TokenKind::EndOfFile => break,
            _ => {}
        }
    }
    */

    let mut parser = Parser::new(&args[1]);
    let file_ast = parser.parse_file();
    println!("{:#?}", file_ast);
}
