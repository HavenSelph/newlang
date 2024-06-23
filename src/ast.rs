use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::span::Span;


pub struct AST {
    span: Span,
    kind: ASTKind
}

impl AST {
    pub fn new(span: Span, kind: ASTKind) -> Self {
        AST {
            span,
            kind
        }
    }
}

pub enum ASTKind {
    StringLiteral(String),
    IntegerLiteral(isize),
    FloatLiteral(f64),
    Add(Rc<AST>, Rc<AST>)
}

impl Display for AST {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ASTKind::StringLiteral(val) => write!(f, "{:?}", val),
            ASTKind::IntegerLiteral(val) => write!(f, "{}", val),
            ASTKind::FloatLiteral(val) => write!(f, "{}", val),
            ASTKind::Add(lhs, rhs) => write!(f, "{} + {}", lhs, rhs)
        }
    }
}