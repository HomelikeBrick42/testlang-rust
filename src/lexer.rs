pub use crate::token::*;

pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: String) -> Lexer {
        Lexer {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn current(&self) -> char {
        *self.source.get(self.position).unwrap_or(&'\0')
    }

    fn next_char(&mut self) -> char {
        let current = &self.current();
        self.position += 1;
        *current
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            let start_position = self.position;
            let start_line = self.line;
            let start_column = self.column;

            macro_rules! token {
                ($kind:expr) => {{
                    return Token::new(
                        $kind,
                        start_position,
                        start_line,
                        start_column,
                        self.position - start_position,
                    );
                }};
            }

            macro_rules! match_token {
                ($kind:expr) => {{
                    self.next_char();
                    token!($kind);
                }};

                ($default_kind:expr, $second_char:expr, $second_kind:expr) => {{
                    self.next_char();
                    if self.current() == $second_char {
                        self.next_char();
                        token!($second_kind);
                    }
                    token!($default_kind);
                }};

                ($default_kind:expr, $second_char:expr, $second_kind:expr, $second_char2:expr, $second_kind2:expr) => {{
                    self.next_char();
                    if self.current() == $second_char {
                        self.next_char();
                        token!($second_kind);
                    } else if self.current() == $second_char2 {
                        self.next_char();
                        token!($second_kind2);
                    }
                    token!($default_kind);
                }};
            }

            match self.current() {
                '\0' => token!(TokenKind::EndOfFile),

                ':' => match_token!(TokenKind::Colon),
                ';' => match_token!(TokenKind::Semicolon),
                '(' => match_token!(TokenKind::LParen),
                ')' => match_token!(TokenKind::RParen),
                '{' => match_token!(TokenKind::LBrace),
                '}' => match_token!(TokenKind::RBrace),
                ',' => match_token!(TokenKind::Comma),

                '+' => match_token!(TokenKind::Plus, '=', TokenKind::PlusEquals),
                '-' => match_token!(TokenKind::Minus, '=', TokenKind::MinusEquals, '>', TokenKind::RightArrow),
                '*' => match_token!(TokenKind::Asterisk, '=', TokenKind::AsteriskEquals),
                '/' => match_token!(TokenKind::Slash, '=', TokenKind::SlashEquals),
                '%' => match_token!(TokenKind::Percent, '=', TokenKind::PercentEquals),
                '=' => match_token!(TokenKind::Equals, '=', TokenKind::EqualsEquals),
                '!' => match_token!(TokenKind::ExclamationMark, '=', TokenKind::ExclamationMarkEquals),

                ' ' | '\n' | '\r' | '\t' => {
                    self.next_char();
                    continue
                }

                // TODO: Allow any utf8 letter?
                'A'..='Z' | 'a'..='z' | '_' => {
                    let mut identifier = String::new();

                    loop {
                        match self.current() {
                            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                                identifier.push(self.next_char());
                            }

                            _ => break,
                        }
                    }

                    token!(TokenKind::Identifier(identifier))
                }

                '0'..='9' => {
                    fn char_to_int(chr: char) -> u64 {
                        match chr {
                            '0'..='9' => chr as u64 - '0' as u64,
                            'A'..='Z' => chr as u64 - 'A' as u64,
                            'a'..='z' => chr as u64 - 'a' as u64,
                            _ => panic!("Unexpected character"),
                        }
                    }

                    let mut int_value = 0;

                    if self.current() == '0' {
                        self.next_char();
                    }

                    let base = match self.current() {
                        'x' | 'X' => {
                            self.next_char();
                            16
                        }

                        'b' | 'B' => {
                            self.next_char();
                            2
                        }

                        _ => 10,
                    };

                    loop {
                        match self.current() {
                            '0'..='9' | 'A'..='Z' | 'a'..='z' => {
                                let value = char_to_int(self.current());

                                if value >= base {
                                    return Token::new(
                                        TokenKind::Error(String::from("Digit grater than base")),
                                        self.position,
                                        self.line,
                                        self.column,
                                        1,
                                    );
                                }

                                int_value *= base;
                                int_value += value;

                                self.next_char();
                            }

                            '.' => {
                                if base != 10 {
                                    return Token::new(
                                        TokenKind::Error(String::from("Float literal must be base 10")),
                                        self.position,
                                        self.line,
                                        self.column,
                                        1,
                                    );
                                }

                                self.next_char();

                                let mut float_value = int_value as f64;
                                let mut denominator = 1;

                                loop {
                                    match self.current() {
                                        '0'..='9' | 'A'..='Z' | 'a'..='z' => {
                                            let value = char_to_int(self.current());

                                            if value >= base {
                                                return Token::new(
                                                    TokenKind::Error(String::from("Digit grater than base")),
                                                    self.position,
                                                    self.line,
                                                    self.column,
                                                    1,
                                                );
                                            }

                                            denominator *= base;
                                            float_value += value as f64 / denominator as f64;

                                            self.next_char();
                                        }

                                        '.' => {
                                            return Token::new(
                                                TokenKind::Error(String::from("Cannot have multiple '.' in float literal")),
                                                self.position,
                                                self.line,
                                                self.column,
                                                1,
                                            );
                                        }

                                        '_' => {
                                            self.next_char();
                                            continue
                                        }

                                        _ => break,
                                    }
                                }

                                token!(TokenKind::Float(float_value))
                            }

                            '_' => {
                                self.next_char();
                                continue
                            }

                            _ => break,
                        }
                    }

                    token!(TokenKind::Integer(int_value))
                }

                _ => match_token!(TokenKind::Error(String::from("Unknown character"))),
            }
        }
    }
}
