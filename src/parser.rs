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
        let source = if let Ok(source) = std::fs::read_to_string(path) {
            source
        } else {
            panic!("Unable to open '{}'", path);
        };

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

    pub fn parse(&mut self) -> Rc<RefCell<Ast>> {
        Rc::new(RefCell::new(Ast::File(self.parse_file((Option::None, Option::None)))))
    }

    fn parse_file(&mut self, parent_data: ParentData) -> Rc<RefCell<AstFile>> {
        let file = Rc::new(RefCell::new(AstFile {
            parent_data: parent_data.clone(),
            file_path: self.file_path.clone(),
            source: self.source.clone(),
            scope: Rc::new(RefCell::new(AstScope {
                parent_data: (Option::None, Option::None),
                statements: Vec::new(),
            })),
        }));

        file.borrow_mut().scope.borrow_mut().parent_data = (Option::Some(Rc::downgrade(&file.clone())), parent_data.1.clone());

        let data = file.borrow().scope.borrow().parent_data.clone();
        while self.current.kind != TokenKind::EndOfFile {
            file.borrow_mut().scope.borrow_mut().statements.push(Rc::new(RefCell::new(self.parse_statement(data.clone()))));
        }

        file
    }

    fn parse_scope(&mut self, parent_data: ParentData) -> Rc<RefCell<AstScope>> {
        if self.current.kind != TokenKind::LBrace {
            panic!("Expected '{{' got {:?}", self.current);
        }
        self.next_token();

        let scope = Rc::new(RefCell::new(AstScope {
            parent_data: parent_data.clone(),
            statements: Vec::new(),
        }));

        while self.current.kind != TokenKind::RBrace {
            scope.borrow_mut().statements.push(Rc::new(RefCell::new(self.parse_statement((parent_data.0.clone(), Option::Some(Rc::downgrade(&scope.clone())))))));
        }

        if self.current.kind != TokenKind::RBrace {
            panic!("Expected '}}' got {:?}", self.current);
        }
        self.next_token();

        scope
    }

    fn parse_statement(&mut self, parent_data: ParentData) -> AstStatement {
        match self.current.kind {
            TokenKind::Semicolon => {
                self.next_token();
                self.parse_statement(parent_data)
            }

            TokenKind::LBrace => {
                AstStatement::Scope(self.parse_scope(parent_data))
            }

            _ => {
                let expression = self.parse_expression(parent_data.clone());

                match self.current.kind {
                    TokenKind::Colon => {
                        self.next_token();

                        let name = if let AstExpression::Name(token) = expression {
                            token.borrow().token.clone()
                        } else {
                            panic!("Expected name before ':'");
                        };

                        let type_ = if self.current.kind != TokenKind::Colon && self.current.kind != TokenKind::Equals {
                            Option::Some(self.parse_type(parent_data.clone()))
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
                            panic!("Expected ':' or '=' got {:?}", self.current);
                        };

                        let value = if self.current.kind != TokenKind::Semicolon {
                            Option::Some(self.parse_expression(parent_data.clone()))
                        } else {
                            Option::None
                        };

                        if let Option::Some(expression) = &value {
                            if !matches!(expression, AstExpression::Procedure(_)) {
                                if self.current.kind != TokenKind::Semicolon {
                                    panic!("Expected ';' got {:?}", self.current);
                                }
                                self.next_token();
                            }
                        } else {
                            if self.current.kind != TokenKind::Semicolon {
                                panic!("Expected ';' got {:?}", self.current);
                            }
                            self.next_token();
                        }

                        if matches!(value, Option::None) & &matches!(type_, Option::None) {
                            panic!("Cannot have a declaration with nether type nor value");
                        }

                        AstStatement::Declaration(Rc::new(RefCell::new(
                            AstDeclaration {
                                parent_data: parent_data.clone(),
                                name,
                                type_: Rc::new(RefCell::new(type_)),
                                value: Rc::new(RefCell::new(value)),
                                constant,
                            }
                        )))
                    }

                    TokenKind::PlusEquals |
                    TokenKind::MinusEquals |
                    TokenKind::AsteriskEquals |
                    TokenKind::SlashEquals |
                    TokenKind::PercentEquals => {
                        let operator = self.next_token();
                        let right = self.parse_expression(parent_data.clone());

                        if self.current.kind != TokenKind::Semicolon {
                            panic!("Expected ';' got {:?}", self.current);
                        }
                        self.next_token();

                        AstStatement::Assignment(Rc::new(RefCell::new(
                            AstAssignment {
                                parent_data: parent_data.clone(),
                                left: Rc::new(RefCell::new(expression)),
                                operator,
                                right: Rc::new(RefCell::new(right)),
                            }
                        )))
                    }

                    _ => {
                        if self.current.kind != TokenKind::Semicolon {
                            panic!("Expected ';' got {:?}", self.current);
                        }
                        self.next_token();

                        AstStatement::Expression(Rc::new(RefCell::new(expression)))
                    },
                }
            }
        }
    }

    fn parse_type(&mut self, parent_data: ParentData) -> AstType {
        match self.current.kind {
            TokenKind::Identifier(_) => {
                AstType::Name(Rc::new(RefCell::new(
                    AstName {
                        parent_data: parent_data.clone(),
                        token: self.next_token()
                    }
                )))
            }

            _ => panic!("Unexpected {:?}", self.current),
        }
    }

    fn parse_expression(&mut self, parent_data: ParentData) -> AstExpression {
        self.parse_binary_expression(0, parent_data)
    }

    fn parse_procedure(&mut self, first_arg_name: Option<AstName>, parent_data: ParentData) -> AstExpression {
        let arguments = if first_arg_name.is_none() {
            Vec::new()
        } else {
            let mut args = Vec::new();

            let first_arg_type = if self.current.kind != TokenKind::Equals {
                Option::Some(self.parse_type(parent_data.clone()))
            } else {
                Option::None
            };

            let first_arg_value = if self.current.kind != TokenKind::Comma && self.current.kind != TokenKind::RParen {
                if self.current.kind != TokenKind::Equals {
                    panic!("Expected '=' got {:?}", self.current);
                }
                Option::Some(self.parse_expression(parent_data.clone()))
            } else {
                Option::None
            };

            if matches!(first_arg_value, Option::None) && matches!(first_arg_type, Option::None) {
                panic!("Cannot have a procedure argument with nether type nor value");
            }

            args.push(Rc::new(RefCell::new(AstDeclaration {
                parent_data: parent_data.clone(),
                name: first_arg_name.unwrap().token,
                type_: Rc::new(RefCell::new(first_arg_type)),
                value: Rc::new(RefCell::new(first_arg_value)),
                constant: false,
            })));

            while self.current.kind != TokenKind::RParen {
                if self.current.kind != TokenKind::Comma {
                    break;
                }
                self.next_token();

                let name = self.next_token();
                if !matches!(name.kind, TokenKind::Identifier(_)) {
                    panic!("Expected name got {:?}", name);
                }

                if self.current.kind != TokenKind::Colon {
                    panic!("Expected ':' got {:?}", self.current);
                }
                self.next_token();

                let type_ = if self.current.kind != TokenKind::Equals {
                    Option::Some(self.parse_type(parent_data.clone()))
                } else {
                    Option::None
                };

                let value = if self.current.kind != TokenKind::Comma {
                    if self.current.kind != TokenKind::Equals {
                        panic!("Expected '=' got {:?}", self.current);
                    }
                    Option::Some(self.parse_expression(parent_data.clone()))
                } else {
                    Option::None
                };

                if matches!(value, Option::None) && matches!(type_, Option::None) {
                    panic!("Cannot have a procedure argument with nether type nor value");
                }

                args.push(Rc::new(RefCell::new(AstDeclaration {
                    parent_data: parent_data.clone(),
                    name,
                    type_: Rc::new(RefCell::new(type_)),
                    value: Rc::new(RefCell::new(value)),
                    constant: false,
                })));
            }

            if self.current.kind != TokenKind::RParen {
                panic!("Expected ')' got {:?}", self.current);
            } else {
                self.next_token();
            }

            args
        };

        let return_type = if self.current.kind == TokenKind::RightArrow {
            self.next_token();
            Option::Some(self.parse_type(parent_data.clone()))
        } else {
            Option::None
        };

        let scope = self.parse_scope(parent_data.clone());

        AstExpression::Procedure(Rc::new(RefCell::new(
            AstProcedure {
                parent_data: parent_data.clone(),
                arguments,
                return_type: Rc::new(RefCell::new(return_type)),
                scope,
            }
        )))
    }

    fn parse_primary_expression(&mut self, parent_data: ParentData) -> AstExpression {
        match self.current.kind {
            TokenKind::Identifier(_) => AstExpression::Name(Rc::new(RefCell::new(
                AstName {
                    parent_data: parent_data.clone(),
                    token: self.next_token()
                }
            ))),

            TokenKind::Integer(_) |
            TokenKind::Float(_) => AstExpression::Literal(Rc::new(RefCell::new(
                AstLiteral {
                    parent_data: parent_data.clone(),
                    token: self.next_token()
                }
            ))),

            TokenKind::LParen => {
                self.next_token();
                if self.current.kind == TokenKind::RParen {
                    self.next_token();
                    return self.parse_procedure(Option::None, parent_data.clone());
                }
                let expression = self.parse_expression(parent_data.clone());
                if self.current.kind == TokenKind::Colon {
                    if let AstExpression::Name(name) = expression {
                        self.next_token();
                        return self.parse_procedure(Option::Some((*name.borrow()).clone()), parent_data.clone());
                    } else  {
                        panic!("Expected name");
                    }
                } else if self.current.kind != TokenKind::RParen {
                    panic!("Expected '(' got {:?}", self.current);
                }
                self.next_token();
                expression
            }

            _ => panic!("Unexpected token {:?}", self.current),
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

    fn parse_binary_expression(&mut self, parent_precedence: u64, parent_data: ParentData) -> AstExpression {
        let unary_precedence = Parser::unary_operator_precedence(&self.current);
        let mut left = if unary_precedence > parent_precedence {
            let operator = self.next_token();
            let operand = self.parse_binary_expression(unary_precedence, parent_data.clone());
            AstExpression::Unary(Rc::new(RefCell::new(
                AstUnary {
                    parent_data: parent_data.clone(),
                    operator,
                    operand: Rc::new(RefCell::new(operand)),
                }
            )))
        } else {
            self.parse_primary_expression(parent_data.clone())
        };

        loop {
            let precedence = Parser::binary_operator_precedence(&self.current);
            if precedence == 0 || precedence <= parent_precedence {
                break;
            }

            let operator = self.next_token();
            let right = self.parse_binary_expression(precedence, parent_data.clone());
            left = AstExpression::Binary(Rc::new(RefCell::new(
                AstBinary {
                    parent_data: parent_data.clone(),
                    left: Rc::new(RefCell::new(left)),
                    operator,
                    right: Rc::new(RefCell::new(right)),
                }
            )));
        }

        left
    }
}
