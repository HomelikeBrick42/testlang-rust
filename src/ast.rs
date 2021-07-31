pub use crate::token::*;
pub use std::rc::*;
pub use std::cell::*;

pub type ParentData = (Option<Weak<RefCell<AstFile>>>, Option<Weak<RefCell<AstScope>>>);

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Ast {
    File(Rc<RefCell<AstFile>>),
    Statement(Rc<RefCell<AstStatement>>),
}

#[derive(Clone, Debug)]
pub struct AstFile {
    pub parent_data: ParentData,
    pub file_path: String,
    pub source: String,
    pub scope: Rc<RefCell<AstScope>>,
}

#[derive(Clone, Debug)]
pub enum AstStatement {
    Expression(Rc<RefCell<AstExpression>>),
    Scope(Rc<RefCell<AstScope>>),
    Declaration(Rc<RefCell<AstDeclaration>>),
    Assignment(Rc<RefCell<AstAssignment>>),
}

#[derive(Clone, Debug)]
pub struct AstScope {
    pub parent_data: ParentData,
    pub statements: Vec<Rc<RefCell<AstStatement>>>,
}

#[derive(Clone, Debug)]
pub struct AstDeclaration {
    pub parent_data: ParentData,
    pub name: Token,
    pub type_: Rc<RefCell<Option<AstType>>>,
    pub value: Rc<RefCell<Option<AstExpression>>>,
    pub constant: bool,
}

#[derive(Clone, Debug)]
pub struct AstAssignment {
    pub parent_data: ParentData,
    pub left: Rc<RefCell<AstExpression>>,
    pub operator: Token,
    pub right: Rc<RefCell<AstExpression>>,
}

#[derive(Clone, Debug)]
pub enum AstExpression {
    Procedure(Rc<RefCell<AstProcedure>>),
    Name(Rc<RefCell<AstName>>),
    Literal(Rc<RefCell<AstLiteral>>),
    Unary(Rc<RefCell<AstUnary>>),
    Binary(Rc<RefCell<AstBinary>>),
}

#[derive(Clone, Debug)]
pub struct AstProcedure {
    pub parent_data: ParentData,
    pub arguments: Vec<Rc<RefCell<AstDeclaration>>>,
    pub return_type: Rc<RefCell<Option<AstType>>>,
    pub scope: Rc<RefCell<AstScope>>,
}

#[derive(Clone, Debug)]
pub struct AstName {
    pub parent_data: ParentData,
    pub token: Token,
}

#[derive(Clone, Debug)]
pub struct AstLiteral {
    pub parent_data: ParentData,
    pub token: Token,
}

#[derive(Clone, Debug)]
pub struct AstUnary {
    pub parent_data: ParentData,
    pub operator: Token,
    pub operand: Rc<RefCell<AstExpression>>,
}

#[derive(Clone, Debug)]
pub struct AstBinary {
    pub parent_data: ParentData,
    pub left: Rc<RefCell<AstExpression>>,
    pub operator: Token,
    pub right: Rc<RefCell<AstExpression>>,
}

#[derive(Clone, Debug)]
pub enum AstType {
    Name(Rc<RefCell<AstTypeName>>),
}

#[derive(Clone, Debug)]
pub struct AstTypeName {
    pub parent_data: ParentData,
    pub name: Token,
}
