use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::{Iter};
use ariadne::{Color, Label};
use crate::ast::{AST, ASTKind};
use crate::token::{Token, TokenKind};
use crate::error::{ErrorReport, ErrorReportKind, Result, ResultErrorless};
use crate::span::Span;

pub struct Parser<'a> {
    current: RefToken<'a>,
    pub had_error: bool,
    tokens: Iter<'a, Token<'a>>,
    reports: Rc<RefCell<Vec<ErrorReport>>>
}

type RefToken<'a> = &'a Token<'a>;

impl<'a> Parser<'a> {
    pub fn new(tokens_vec: &'a [Token], reports: Rc<RefCell<Vec<ErrorReport>>>) -> Self {
        let mut tokens = tokens_vec.iter();
        Parser {
            current: tokens.next().expect("EOF Token doesn't exist."),
            had_error: false,
            tokens,
            reports
        }
    }

    fn push_report(&mut self, report: ErrorReport) {
        self.had_error = true;
        self.reports.borrow_mut().push(report)
    }

    fn advance(&mut self) -> RefToken {
        self.current = self.tokens.next().expect("EOF Token skipped.");
        self.current
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<RefToken> {
        let token = self.current;
        if token.kind == kind {
            self.advance();
            Ok(token)
        } else if token.kind.clone() == TokenKind::EOF {
            let e = ErrorReport::new(ErrorReportKind::Custom, token.span.clone(), "Unexpected EOF".to_string())
                .with_label(Label::new(token.span.clone()).with_message(message).with_color(Color::Red));
            Err(e)
        } else {
            let e = ErrorReport::new(ErrorReportKind::UnexpectedToken, token.span.clone(), format!("got {:?}", token.kind))
                .with_label(Label::new(token.span.clone()).with_message(message).with_color(Color::Red));
            Err(e)
        }
    }

    fn consume_line_end(&mut self) -> Result<()> {
        match self.current.kind {
            TokenKind::SemiColon => Ok(()),
            TokenKind::EOF => Ok(()),
            _ => {
                let e = ErrorReport::new(ErrorReportKind::UnexpectedToken, self.current.span.clone(), format!("Expected end of line but got {:?}", self.current.kind))
                    .with_label(Label::new(self.current.span.clone()).with_color(Color::Red));
                Err(e)
            }
        }
    }

    pub fn parse(&mut self) -> Option<Rc<AST>> {
        match self.parse_atom() {
            Ok(node) => Some(node),
            Err(error) => {
                self.push_report(error);
                None
            }
        }
    }

    pub fn parse_atom(&mut self) -> Result<Rc<AST>> {
        match self.current {
            Token { kind: TokenKind::StringLiteral, span, text, .. } => {
                self.advance();
                Ok(Rc::new(AST::new(span.clone(), ASTKind::StringLiteral(text.to_string()))))
            }
            Token { kind: TokenKind::EOF, span, .. } => {
                let e = ErrorReport::new(ErrorReportKind::Custom, span.clone(), "Unexpected EOF".to_string());
                Err(e)
            }
            Token { kind, span, .. } => {
                let e = ErrorReport::new(ErrorReportKind::UnexpectedToken, span.clone(), format!("{:?}", kind))
                    .with_label(Label::new(span.clone()).with_color(Color::Red));
                Err(e)
            }
        }
    }
}