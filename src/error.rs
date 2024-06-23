use std::cmp::Ordering;
use std::fmt::format;
use ariadne::{Color, Config, LabelAttach, Report, ReportBuilder, ReportKind};
use clap::ValueEnum;
use crate::span::{Span};

pub type Result<T> = std::result::Result<T, ErrorReport>;
pub type ResultErrorless<T> = std::result::Result<T, ()>;


#[derive(PartialOrd, PartialEq, Copy, Clone, Debug, ValueEnum)]
pub enum ErrorLevel {
    /// Show no error messages at all.
    Silent,
    /// Show only minimal context.
    Compact,
    /// Show all context.
    Normal,
}

pub struct ErrorReport {
    span: Span,
    title: String,
    labels: Vec<ariadne::Label<Span>>
}

impl ErrorReport {
    pub fn new(kind: ErrorReportKind, span: Span, message: String) -> Self {
        let title = if kind != ErrorReportKind::Custom {
            format!("{:?}: {}", kind, message)
        } else {
            message
        };
        ErrorReport {
            span,
            title,
            labels: Vec::new()
        }
    }

    pub fn with_label(mut self, label: ariadne::Label<Span>) -> Self {
        self.labels.push(label);
        self
    }

    pub fn to_ariadne_report(&self, level: ErrorLevel) -> Report<Span> {
        match level {
            ErrorLevel::Silent => unreachable!("Cannot make a silent ariadne report."),
            ErrorLevel::Compact => {
                let report_kind = ReportKind::Custom(Box::leak(format!("{} Error", self.span.clone()).into_boxed_str()), Color::Red);
                Report::build(report_kind, self.span.filename.clone(), self.span.start)
                .with_message(self.title.clone())
                .with_config(Config::default().with_compact(true))
                .finish()
            },
            ErrorLevel::Normal => Report::build(ReportKind::Error, self.span.filename.clone(), self.span.start)
                .with_message(self.title.clone())
                .with_labels(self.labels.clone())
                .finish(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorReportKind {
    // Errors
    SyntaxError,
    UnexpectedCharacter,
    UnexpectedToken,
    DidYouMean,
    Custom
}

// This was the old code for doing errors before I decided an error struct was necessary.
//
// // These were intended to be macros, and I'm sure a nice macro to do this is possible,
// // but I see no issue with just having them as inlines.
// #[inline]
// pub fn report<'a>(report_kind: ReportKind<'a>, span: Span, message: &str) -> ReportBuilder<'a> {
//     Report::<Span>::build(report_kind, span.filename, span.start).with_message(message)
// }
//
// #[inline]
// pub fn error<'a>(span: Span, kind: ErrorKind, title: &str) -> ReportBuilder<'a> {
//     report(ReportKind::Error, span, format!("{kind}: {title}").as_str())
// }
//
// pub fn warn<'a>(span: Span, title: &str) -> ReportBuilder<'a> {
//     report(ReportKind::Warning, span, title)
// }

// pub fn note<'a>(span: Span, title: &str) -> ReportBuilder<'a> {
//     report(ReportKind::Advice, span, title)
// }