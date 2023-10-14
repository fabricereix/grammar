extern crate grammar;

mod exit;
mod options;
mod pos;

use exit::*;
use grammar::{format_html, parse};
use options::*;
use pos::Pos;
use std::fs::File;
use std::io;
use std::io::Read;

fn get_content(options: &CliOptions) -> String {
    match options.input_file.clone() {
        None => {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content).unwrap();
            content
        }
        Some(input_file) => {
            let mut s = String::new();
            let mut f = File::open(input_file).expect("Unable to open file");
            f.read_to_string(&mut s).expect("Unable to read string");
            s
        }
    }
}

fn main() {
    let options = parse_options();
    let content = get_content(&options);
    let g = match parse(&content) {
        Ok(value) => value,
        Err(errors) => {
            for error in errors {
                let pos = find_position(&content, error.span.start);
                eprintln!("{}:{}: {}", pos.line, pos.column, error.message);
            }
            ExitCode::ErrorParsing.exit()
        }
    };

    let errors_validate = g.validate();
    if !errors_validate.is_empty() {
        for error in g.validate() {
            let pos = find_position(&content, error.span.start);
            eprintln!("{}:{}: {}", pos.line, pos.column, error.message);
        }
        ExitCode::ErrorValidation.exit()
    }

    println!(
        "{}",
        format_html(&g, &content, &options.section_header, options.section_id)
    );
    ExitCode::Success.exit()
}

pub fn find_position(s: &str, offset: usize) -> Pos {
    debug_assert!(offset < s.len());
    let positions = Pos::all(s);

    positions
        .get(offset)
        .expect("offset smaller that size")
        .clone()
}
