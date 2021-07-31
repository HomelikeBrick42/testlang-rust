pub use crate::token::*;

pub type ParentData<'a> = (Option<&'a AstFile<'a>>, Option<&'a AstScope<'a>>);

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Ast<'a> {
    File(Box<AstFile<'a>>),
    Statement(Box<AstStatement<'a>>),
}

#[derive(Clone, Debug)]
pub struct AstFile<'a> {
    pub parent_data: ParentData<'a>,
    pub file_path: String,
    pub source: String,
    pub scope: AstScope<'a>,
}

#[derive(Clone, Debug)]
pub enum AstStatement<'a> {
    Expression(Box<AstExpression<'a>>),
    Scope(Box<AstScope<'a>>),
    Declaration(Box<AstDeclaration<'a>>),
    Assignment(Box<AstAssignment<'a>>),
}

#[derive(Clone, Debug)]
pub struct AstScope<'a> {
    pub parent_data: ParentData<'a>,
    pub statements: Vec<AstStatement<'a>>,
}

#[derive(Clone, Debug)]
pub struct AstDeclaration<'a> {
    pub parent_data: ParentData<'a>,
    pub name: Token,
    pub type_: Option<AstType<'a>>,
    pub value: Option<AstExpression<'a>>,
    pub constant: bool,
}

#[derive(Clone, Debug)]
pub struct AstAssignment<'a> {
    pub parent_data: ParentData<'a>,
    pub left: AstExpression<'a>,
    pub operator: Token,
    pub right: AstExpression<'a>,
}

#[derive(Clone, Debug)]
pub enum AstExpression<'a> {
    Procedure(Box<AstProcedure<'a>>),
    Name(Box<AstName<'a>>),
    Literal(Box<AstLiteral<'a>>),
    Unary(Box<AstUnary<'a>>),
    Binary(Box<AstBinary<'a>>),
}

#[derive(Clone, Debug)]
pub struct AstProcedure<'a> {
    pub parent_data: ParentData<'a>,
    pub arguments: Vec<AstDeclaration<'a>>,
    pub return_type: Option<AstType<'a>>,
    pub scope: AstScope<'a>,
}

#[derive(Clone, Debug)]
pub struct AstName<'a> {
    pub parent_data: ParentData<'a>,
    pub token: Token,
}

#[derive(Clone, Debug)]
pub struct AstLiteral<'a> {
    pub parent_data: ParentData<'a>,
    pub token: Token,
}

#[derive(Clone, Debug)]
pub struct AstUnary<'a> {
    pub parent_data: ParentData<'a>,
    pub operator: Token,
    pub operand: AstExpression<'a>,
}

#[derive(Clone, Debug)]
pub struct AstBinary<'a> {
    pub parent_data: ParentData<'a>,
    pub left: AstExpression<'a>,
    pub operator: Token,
    pub right: AstExpression<'a>,
}

#[derive(Clone, Debug)]
pub enum AstType<'a> {
    Name(Box<AstTypeName<'a>>),
}

#[derive(Clone, Debug)]
pub struct AstTypeName<'a> {
    pub parent_data: ParentData<'a>,
    pub name: Token,
}
