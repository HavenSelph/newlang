#![allow(unused)]
extern crate core;

use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::process::exit;
use std::sync::{Arc};
use ariadne::{Report, Source};
use clap::Parser as ArgParser;

pub mod lexer;
pub mod span;
pub mod token;
mod error;
mod parser;
mod ast;


use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::error::{ErrorLevel, ErrorReport};


#[derive(ArgParser, Debug)]
#[command(version, about = "Haven's interpreter", version="0.0.1")]
struct Args {
    #[arg(default_value = None)]
    filename: Option<String>,
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long, value_enum, default_value_t=ErrorLevel::Normal)]
    error_level: ErrorLevel
}

fn interpret(debug: bool, filename: Arc<str>, contents: &str, reports: Rc<RefCell<Vec<ErrorReport>>>) -> i32 {
    let mut lexer = Lexer::new(filename.clone(), contents, reports.clone());
    lexer.lex_tokens();
    if debug {
        for (i, token) in lexer.tokens.iter().enumerate() {
            println!("{}: {}", i, token);
        }
    }
    if lexer.had_error { return 64; }

    let mut parser = Parser::new(&lexer.tokens, reports.clone());
    let Some(ast) = parser.parse() else { return 64; };
    if debug { println!("{}", ast) }
    if parser.had_error { return 64; }

    // Interpret!
    unimplemented!("Reached interpretation step, not yet finished.");
}

fn print_reports(level: ErrorLevel, filename: Arc<str>, contents: &String, reports: Rc<RefCell<Vec<ErrorReport>>>) {
    let silent = level == ErrorLevel::Silent;
    let cache = (filename, Source::from(contents));
    let mut emitted_errors: usize = 0;
    for report in reports.borrow().iter() {
        if !silent {
            report.to_ariadne_report(level).eprint(cache.clone()).unwrap();
        }
        emitted_errors += 1;
    };
    if !silent {
        if emitted_errors == 1 {
            eprintln!("Emitted {} error.", emitted_errors);
        } else if emitted_errors > 0 {
            eprintln!("Emitted {} errors.", emitted_errors);
        }
    }
}

fn repl(_debug: bool) {
    unimplemented!("Repl is not implemented.");
}

fn main() {
    let args = Args::parse();

    if args.filename.is_some() {
        let mut reports = Rc::new(RefCell::new(Vec::<ErrorReport>::new()));
        let arc_filename: Arc<str> = Arc::from(args.filename.unwrap());

        let mut contents = String::new();
        File::open(arc_filename.deref()).unwrap().read_to_string(&mut contents).unwrap();

        let code = interpret(args.debug, arc_filename.clone(), &contents, reports.clone());
        print_reports(args.error_level, arc_filename, &contents, reports);
        exit(code);
    } else {
        repl(args.debug)
    }
}
