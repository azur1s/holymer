use lexer::Token;
use chumsky::prelude::Simple;
use ariadne::{Report, ReportKind, Label, Source, Color, Fmt};

#[derive(Debug)]
pub struct Diagnostics {
    pub errors: Vec<Kind>,
}

#[derive(Debug)]
pub enum Kind {
    LexError(Simple<char>),
    ParseError(Simple<Token>),
    LoweringError(hir::LoweringError),
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn add_lex_error(&mut self, error: Simple<char>) {
        self.errors.push(Kind::LexError(error));
    }

    pub fn add_parse_error(&mut self, error: Simple<Token>) {
        self.errors.push(Kind::ParseError(error));
    }

    pub fn add_lowering_error(&mut self, error: hir::LoweringError) {
        self.errors.push(Kind::LoweringError(error));
    }

    pub fn display(&self, src: String) {
        let lex_error = self.errors.iter().filter_map(|kind| match kind {
            Kind::LexError(error) => Some(error.clone()), // Using clone() to remove reference
            _ => None,
        });
        let parse_error = self.errors.iter().filter_map(|kind| match kind {
            Kind::ParseError(error) => Some(error.clone()), // Same case as above
            _ => None,
        });

        lex_error.into_iter()
        .map(|e| e.map(|e| e.to_string()))
        .chain(parse_error.into_iter().map(|e| e.map(|tok| tok.to_string())))
        .for_each(|e| {
            let report = Report::build(ReportKind::Error, (), e.span().start);

            let report = match e.reason() {
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                .with_message(format!(
                    "Unclosed delimiter {}",
                    delimiter.fg(Color::Yellow)
                ))
                .with_label(
                    Label::new(span.clone())
                    .with_message(format!(
                        "Expected closing delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_color(Color::Yellow)
                )
                .with_label(
                    Label::new(e.span())
                    .with_message(format!(
                        "Must be closed before this {}",
                        e.found()
                        .unwrap_or(&"end of file".to_string())
                        .fg(Color::Red)
                    ))
                    .with_color(Color::Red)
                ),

                chumsky::error::SimpleReason::Unexpected => report
                .with_message(format!(
                    "{}, expected {}",

                    if e.found().is_some() { "Unexpected token in input" }
                    else { "Unexpected end of input" },

                    if e.expected().len() == 0 { "something else".to_string().fg(Color::Green) }
                    else {
                        e.expected()
                        .map(|expected| match expected {
                            Some(expected) => expected.to_string(),
                            None => "end of input".to_string()
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                        .fg(Color::Green)
                    }
                ))
                .with_label(
                    Label::new(e.span())
                    .with_message(format!(
                        "Unexpected token {}",
                        e.found()
                        .unwrap_or(&"EOF".to_string())
                        .fg(Color::Red)
                    ))
                    .with_color(Color::Red)
                ),
                _ => {
                    println!("{:?}", e);
                    todo!();
                }
            };

            report.finish().print(Source::from(&src)).unwrap();
        }); // End errors reporting

        let lower_error = self.errors.iter().filter_map(|kind| match kind {
            Kind::LoweringError(error) => Some(error.clone()),
            _ => None,
        });

        lower_error.into_iter()
        .for_each(|e| {
            let span = &e.span;
            let message = &e.message;

            let report = Report::build(ReportKind::Error, (), span.start)
                .with_message(
                    format!("{}", message)
                )
                .with_label(
                    Label::new(span.clone())
                    .with_message(
                        format!("{}", message)
                    )
                    .with_color(Color::Red)
                );

            if let Some(note) = &e.note {
                report
                .with_note(note)
                .finish().print(Source::from(&src)).unwrap();
            } else {
                report.finish().print(Source::from(&src)).unwrap();
            }
        });
    }
}
