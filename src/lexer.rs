use std::cell::RefCell;
use std::iter::Peekable;
use std::ops::Index;
use std::rc::Rc;
use std::str::Chars;
use std::sync::{Arc};
use ariadne::{Color, Label};
use crate::error::{ResultErrorless, ErrorReport, ErrorReportKind};
use crate::span::{Span};
use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    filename: Arc<str>,
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    current: Option<char>,
    index: usize,
    pub had_error: bool,
    pub tokens: Vec<Token<'a>>,
    reports: Rc<RefCell<Vec<ErrorReport>>>
}

impl<'a> Lexer<'a> {
    pub fn new(filename: Arc<str>, source: &'a str, reports: Rc<RefCell<Vec<ErrorReport>>>) -> Self {
        let mut chars = source.chars().peekable();
        Lexer {
            filename,
            current: chars.next(),
            index: 0,
            source,
            chars,
            had_error: false,
            tokens: Vec::new(),
            reports
        }
    }

    fn peek(&mut self, offset: usize) -> Option<char> { self.chars.peek().cloned() }

    fn span(&self, start: usize, end: usize) -> Span { Span::new(start, end, self.filename.clone()) }

    fn span_at(&self, index: usize) -> Span { Span::location(index, self.filename.clone()) }

    fn span_from(&self, from: usize) -> Span { Span::new(from, self.index, self.filename.clone()) }

    fn advance(&mut self) {
        if self.current.is_some() {
            self.current = self.chars.next();
            self.index += 1;
        }
    }

    fn push(&mut self, mut token: Token<'a>) {
        self.tokens.push(token)
    }

    fn push_simple(&mut self, kind: TokenKind, length: usize) {
        let start = self.index;
        let text = &self.source[self.index..self.index+length];
        for _ in 0..length {
            self.advance();
        }
        self.push(Token::new(kind, self.span_from(start), text))
    }

    fn push_report(&mut self, report: ErrorReport) {
        self.reports.borrow_mut().push(report);
        self.had_error = true;
    }

    pub fn lex_tokens(&mut self) {
        while let Some(char) = self.current {
            let start = self.index;
            match char {
                c if c.is_whitespace() => self.advance(),
                'a'..='z' | 'A'..='Z' | '_' => {
                    while let Some(c) = self.current {
                        match c {
                            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                                self.advance();
                            }
                            _ => break
                        }
                    };
                    let ident = &self.source[start..self.index];
                    let span = self.span(start, self.index-1);
                    let kind = match ident {
                        "let" => TokenKind::Let,
                        _ => TokenKind::Identifier
                    };
                    self.push(Token::new(kind, span, ident))
                }
                '0' if self.peek(1).map_or(false, |c| "box".contains(c)) => {
                    let base = match (char, self.peek(1)) {
                        ('0', Some('b')) => Base::Bin,
                        ('0', Some('o')) => Base::Oct,
                        ('0', Some('x')) => Base::Hex,
                        _ => unreachable!()
                    };
                    self.advance();
                    self.advance();
                    if self.lex_integer(base, start).is_err() {
                        continue;
                    }
                    let num = &self.source[start..self.index];
                    self.push(Token::new(TokenKind::from(base), self.span_from(start), num));
                }
                '0'..='9' => {
                    if self.lex_integer(Base::Dec, start).is_err() {
                        continue;
                    }
                    if let Some('.') = self.current {
                        self.advance();
                        if self.lex_integer(Base::Dec, start).is_err() {
                            continue;
                        }
                        if let Some('.') = self.current {
                            let span = self.span_from(start);
                            let e = ErrorReport::new(ErrorReportKind::SyntaxError, span, "Invalid Float Literal".to_string())
                                .with_label(Label::new(self.span_at(self.index)).with_message("Second fractional indicator").with_color(Color::Red));
                            self.push_report(e);
                            continue;
                        }
                        let num = &self.source[start..self.index];
                        self.push(Token::new(TokenKind::FloatLiteral, self.span_from(start), num));
                        continue;
                    }
                    let num = &self.source[start..self.index];
                    self.push(Token::new(TokenKind::IntegerLiteralDec, self.span_from(start), num));
                },
                '.' => match self.peek(1) {
                    Some('0'..='9') => {
                        self.advance();
                        if self.lex_integer(Base::Dec, start).is_err() {
                            continue;
                        }
                        if let Some('.') = self.current {
                            let span = self.span_from(start);
                            let e = ErrorReport::new(ErrorReportKind::SyntaxError, span, "Invalid Float Literal".to_string())
                                .with_label(Label::new(self.span_at(self.index)).with_message("Second fractional indicator").with_color(Color::Red));
                            self.push_report(e);
                            continue;
                        }
                        self.push(Token::new(TokenKind::FloatLiteral, self.span_from(start), &self.source[start..self.index]));
                    }
                    _ => self.push_simple(TokenKind::Period, 1)
                }
                '/' => match self.peek(1) {
                    Some('/') => {
                        while let Some(char) = self.current {
                            if char == '\n' {
                                break;
                            }
                            self.advance()
                        }
                    }
                    Some('*') => {
                        let mut depth: usize = 1;
                        self.advance();
                        while depth > 0 {
                            let Some(char) = self.current else {
                                let span = self.span_from(start);
                                let e = ErrorReport::new(ErrorReportKind::SyntaxError, span, "Unterminated Multi-Line Comment".to_string())
                                    .with_label(Label::new(self.span(start, start+2)).with_message("Comment started here").with_color(Color::Red));
                                self.push_report(e);
                                break;
                            };
                            match char {
                                '/' if self.peek(1) == Some('*') => depth += 1,
                                '*' if self.peek(1) == Some('/') => {
                                    self.advance();
                                    self.advance();
                                    depth -= 1;
                                },
                                _ => {}
                            }
                            self.advance();
                        }
                    }
                    _ => self.push_simple(TokenKind::Slash, 1)
                }
                ';' => self.push_simple(TokenKind::SemiColon, 1),
                '=' => self.push_simple(TokenKind::Equals, 1),
                _ => {
                    let span = self.span_at(self.index);
                    let e = ErrorReport::new(ErrorReportKind::UnexpectedCharacter, span.clone(), format!("{:?}", self.current.expect("Lexer matched on Some but found None")))
                        .with_label(Label::new(span).with_message("Not a valid character.").with_color(Color::Red));
                    self.push_report(e);
                    self.advance();
                }
            };
        }
        self.push_simple(TokenKind::EOF, 0);
    }

    fn lex_integer(&mut self, base: Base, start: usize) -> ResultErrorless<()> {
        // use slices instead
        while let Some(char) = self.current {
            match (base, char.to_ascii_lowercase()) {
                (Base::Bin, '0'..='1')
                | (Base::Oct, '0'..='7')
                | (Base::Dec, '0'..='9')
                | (Base::Hex, '0'..='9' | 'a'..='f') => {
                    self.advance();
                },
                (_, '0'..='9' | 'a'..='z') => {
                    let span = self.span_from(start);
                    let e = ErrorReport::new(ErrorReportKind::SyntaxError, span.clone(), "Invalid Integer Literal".to_string())
                        .with_label(Label::new(span).with_message(format!("{} integer literal", base.to_string())).with_color(Color::BrightBlue).with_order(1))
                        .with_label(Label::new(self.span_at(self.index)).with_message("Invalid character").with_color(Color::Red));
                    self.push_report(e);
                    // self.advance(); // Is doing this a worse approach to handling this error?
                    return Err(())
                },
                (_, '_') => self.advance(),
                _ => break
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Base {
    Bin,
    Oct,
    Dec,
    Hex,
}

impl Base {
    pub fn to_string(self) -> &'static str {
        match self {
            Base::Bin => "Binary",
            Base::Oct => "Octal",
            Base::Dec => "Decimal",
            Base::Hex => "Hexadecimal"
        }
    }
}

impl From<Base> for TokenKind {
    fn from(value: Base) -> Self {
        match value {
            Base::Bin => Self::IntegerLiteralBin,
            Base::Oct => Self::IntegerLiteralOct,
            Base::Dec => Self::IntegerLiteralDec,
            Base::Hex => Self::IntegerLiteralHex,
        }
    }
}