pub use crate::ast::*;
use crate::lexer::*;

pub struct Parser {
    file_path: String,
    source: String,
    lexer: Lexer,
    current: Token,
}

impl Parser {
    pub fn new(path: &str) -> Parser {
        let source = std::fs::read_to_string(path).expect("Unable to open file!");
        let mut lexer = Lexer::new(source.clone());
        Parser {
            file_path: String::from(path),
            source,
            current: lexer.next_token(),
            lexer,
        }
    }

    fn next_token(&mut self) -> Token {
        let token = self.current.clone();
        self.current = self.lexer.next_token();
        token
    }

    pub fn parse_file(&mut self) -> AstFile {
        let mut statements = Vec::new();

        while self.current.kind != TokenKind::EndOfFile {
            statements.push(self.parse_statement());
        }

        AstFile {
            file_path: self.file_path.clone(),
            source: self.source.clone(),
            scope: AstScope {
                statements,
            },
        }
    }

    fn parse_scope(&mut self) -> AstScope {
        if self.current.kind != TokenKind::LBrace {
            panic!("Expected '{{' got {:?}", self.current.kind);
        }
        self.next_token();

        let mut statements = Vec::new();
        while self.current.kind != TokenKind::RBrace {
            statements.push(self.parse_statement());
        }

        if self.current.kind != TokenKind::RBrace {
            panic!("Expected '}}' got {:?}", self.current.kind);
        }
        self.next_token();

        AstScope {
            statements,
        }
    }

    fn parse_statement(&mut self) -> AstStatement {
        match self.current.kind {
            TokenKind::Semicolon => {
                self.next_token();
                self.parse_statement()
            }

            TokenKind::LBrace => {
                AstStatement::Scope(Box::new(self.parse_scope()))
            }

            _ => {
                let expression = self.parse_expression();

                match self.current.kind {
                    TokenKind::Colon => {
                        self.next_token();

                        let name = if let AstExpression::Name(token) = expression {
                            token.token
                        } else {
                            panic!("Expected name before ':'");
                        };

                        let type_ = if self.current.kind != TokenKind::Colon && self.current.kind != TokenKind::Equals {
                            Option::Some(self.parse_type())
                        } else {
                            Option::None
                        };

                        let constant = if self.current.kind == TokenKind::Equals {
                            self.next_token();
                            false
                        } else if self.current.kind == TokenKind::Colon {
                            self.next_token();
                            true
                        } else {
                            panic!("Expected ':' or '=' got {:?}", self.current.kind);
                        };

                        let value = if self.current.kind != TokenKind::Semicolon {
                            Option::Some(self.parse_expression())
                        } else {
                            Option::None
                        };

                        if let Option::Some(expression) = &value {
                            if !matches!(expression, AstExpression::Procedure(_)) {
                                if self.current.kind != TokenKind::Semicolon {
                                    panic!("Expected ';' got {:?}", self.current.kind);
                                }
                                self.next_token();
                            }
                        } else {
                            if self.current.kind != TokenKind::Semicolon {
                                panic!("Expected ';' got {:?}", self.current.kind);
                            }
                            self.next_token();
                        }

                        if matches!(value, Option::None) & &matches!(type_, Option::None) {
                            panic!("Cannot have a declaration with nether type nor value");
                        }

                        AstStatement::Declaration(Box::new(
                            AstDeclaration {
                                name,
                                type_,
                                value,
                                constant,
                            }
                        ))
                    }

                    TokenKind::PlusEquals |
                    TokenKind::MinusEquals |
                    TokenKind::AsteriskEquals |
                    TokenKind::SlashEquals |
                    TokenKind::PercentEquals => {
                        let operator = self.next_token();
                        let right = self.parse_expression();

                        if self.current.kind != TokenKind::Semicolon {
                            panic!("Expected ';' got {:?}", self.current.kind);
                        }
                        self.next_token();

                        AstStatement::Assignment(Box::new(
                            AstAssignment {
                                left: expression,
                                operator,
                                right,
                            }
                        ))
                    }

                    _ => {
                        if self.current.kind != TokenKind::Semicolon {
                            panic!("Expected ';' got {:?}", self.current.kind);
                        }
                        self.next_token();

                        AstStatement::Expression(Box::from(expression))
                    },
                }
            }
        }
    }

    fn parse_type(&mut self) -> AstType {
        match self.current.kind {
            TokenKind::Identifier(_) => {
                AstType::Name(Box::new(
                    AstTypeName {
                        name: self.next_token()
                    }
                ))
            }

            _ => panic!("Unexpected {:?}", self.current.kind),
        }
    }

    fn parse_expression(&mut self) -> AstExpression {
        self.parse_binary_expression(0)
    }

    fn parse_procedure(&mut self, first_arg_name: Option<AstName>) -> AstExpression {
        let arguments = if first_arg_name.is_none() {
            Vec::new()
        } else {
            let mut args = Vec::new();

            let first_arg_type = if self.current.kind != TokenKind::Equals {
                Option::Some(self.parse_type())
            } else {
                Option::None
            };

            let first_arg_value = if self.current.kind != TokenKind::Comma && self.current.kind != TokenKind::RParen {
                if self.current.kind != TokenKind::Equals {
                    panic!("Expected '=' got {:?}", self.current.kind);
                }
                Option::Some(self.parse_expression())
            } else {
                Option::None
            };

            if matches!(first_arg_value, Option::None) && matches!(first_arg_type, Option::None) {
                panic!("Cannot have a procedure argument with nether type nor value");
            }

            args.push(AstDeclaration {
                name: first_arg_name.unwrap().token,
                type_: first_arg_type,
                value: first_arg_value,
                constant: false,
            });

            while self.current.kind != TokenKind::RParen {
                if self.current.kind != TokenKind::Comma {
                    break;
                }
                self.next_token();

                let name = self.next_token();
                if !matches!(name.kind, TokenKind::Identifier(_)) {
                    panic!("Expected name got {:?}", name.kind);
                }

                if self.current.kind != TokenKind::Colon {
                    panic!("Expected ':' got {:?}", self.current.kind);
                }
                self.next_token();

                let type_ = if self.current.kind != TokenKind::Equals {
                    Option::Some(self.parse_type())
                } else {
                    Option::None
                };

                let value = if self.current.kind != TokenKind::Comma {
                    if self.current.kind != TokenKind::Equals {
                        panic!("Expected '=' got {:?}", self.current.kind);
                    }
                    Option::Some(self.parse_expression())
                } else {
                    Option::None
                };

                if matches!(value, Option::None) && matches!(type_, Option::None) {
                    panic!("Cannot have a procedure argument with nether type nor value");
                }

                args.push(AstDeclaration {
                    name,
                    type_,
                    value,
                    constant: false,
                });
            }

            if self.current.kind != TokenKind::RParen {
                panic!("Expected ')' got {:?}", self.current.kind);
            } else {
                self.next_token();
            }

            args
        };

        let return_type = if self.current.kind == TokenKind::RightArrow {
            self.next_token();
            Option::Some(self.parse_type())
        } else {
            Option::None
        };

        let scope = self.parse_scope();

        AstExpression::Procedure(Box::new(
            AstProcedure {
                arguments,
                return_type,
                scope,
            }
        ))
    }

    fn parse_primary_expression(&mut self) -> AstExpression {
        match self.current.kind {
            TokenKind::Identifier(_) => AstExpression::Name(Box::new(
                AstName {
                    token: self.next_token()
                }
            )),

            TokenKind::Integer(_) |
            TokenKind::Float(_) => AstExpression::Literal(Box::new(
                AstLiteral {
                    token: self.next_token()
                }
            )),

            TokenKind::LParen => {
                self.next_token();
                if self.current.kind == TokenKind::RParen {
                    self.next_token();
                    return self.parse_procedure(Option::None);
                }
                let expression = self.parse_expression();
                if self.current.kind == TokenKind::Colon {
                    if let AstExpression::Name(name) = expression {
                        self.next_token();
                        return self.parse_procedure(Option::Some(*name));
                    } else  {
                        panic!("Expected name");
                    }
                } else if self.current.kind != TokenKind::RParen {
                    panic!("Expected '(' got {:?}", self.current.kind);
                }
                self.next_token();
                expression
            }

            _ => panic!("Unexpected token {:?}", self.current.kind),
        }
    }

    fn unary_operator_precedence(token: &Token) -> u64 {
        match token.kind {
            TokenKind::Plus => 3,
            TokenKind::Minus => 3,

            _ => 0,
        }
    }

    fn binary_operator_precedence(token: &Token) -> u64 {
        match token.kind {
            TokenKind::Asterisk => 2,
            TokenKind::Slash => 2,
            TokenKind::Percent => 2,

            TokenKind::Plus => 1,
            TokenKind::Minus => 1,

            _ => 0,
        }
    }

    fn parse_binary_expression(&mut self, parent_precedence: u64) -> AstExpression {
        let unary_precedence = Parser::unary_operator_precedence(&self.current);
        let mut left = if unary_precedence > parent_precedence {
            let operator = self.next_token();
            let operand = self.parse_binary_expression(unary_precedence);
            AstExpression::Unary(Box::new(
                AstUnary {
                    operator,
                    operand,
                }
            ))
        } else {
            self.parse_primary_expression()
        };

        loop {
            let precedence = Parser::binary_operator_precedence(&self.current);
            if precedence == 0 || precedence <= parent_precedence {
                break;
            }

            let operator = self.next_token();
            let right = self.parse_binary_expression(precedence);
            left = AstExpression::Binary(Box::new(
                AstBinary {
                    left,
                    operator,
                    right,
                }
            ));
        }

        left
    }
}
