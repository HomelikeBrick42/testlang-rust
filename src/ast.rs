pub use crate::token::*;

#[derive(Clone, Debug)]
pub enum Ast {
    File(Box<AstFile>),
    Statement(Box<AstStatement>),
}

#[derive(Clone, Debug)]
pub struct AstFile {
    pub file_path: String,
    pub source: String,
    pub scope: AstScope,
}

#[derive(Clone, Debug)]
pub enum AstStatement {
    Expression(Box<AstExpression>),
    Scope(Box<AstScope>),
    Declaration(Box<AstDeclaration>),
    Assignment(Box<AstAssignment>),
}

#[derive(Clone, Debug)]
pub struct AstScope {
    pub statements: Vec<AstStatement>,
}

#[derive(Clone, Debug)]
pub struct AstDeclaration {
    pub name: Token,
    pub type_: Option<AstType>,
    pub value: Option<AstExpression>,
    pub constant: bool,
}

#[derive(Clone, Debug)]
pub struct AstAssignment {
    pub left: AstExpression,
    pub operator: Token,
    pub right: AstExpression,
}

#[derive(Clone, Debug)]
pub enum AstExpression {
    Procedure(Box<AstProcedure>),
    Name(Box<AstName>),
    Literal(Box<AstLiteral>),
    Unary(Box<AstUnary>),
    Binary(Box<AstBinary>),
}

#[derive(Clone, Debug)]
pub struct AstProcedure {
    pub arguments: Vec<AstDeclaration>,
    pub return_type: Option<AstType>,
    pub scope: AstScope,
}

#[derive(Clone, Debug)]
pub struct AstName {
    pub token: Token,
}

#[derive(Clone, Debug)]
pub struct AstLiteral {
    pub token: Token,
}

#[derive(Clone, Debug)]
pub struct AstUnary {
    pub operator: Token,
    pub operand: AstExpression,
}

#[derive(Clone, Debug)]
pub struct AstBinary {
    pub left: AstExpression,
    pub operator: Token,
    pub right: AstExpression,
}

#[derive(Clone, Debug)]
pub enum AstType {
    Name(Box<AstTypeName>),
}

#[derive(Clone, Debug)]
pub struct AstTypeName {
    pub name: Token,
}
