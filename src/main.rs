use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::{Parser, prelude::Input};
use self::{parse::parser::{lexer, exprs_parser}, typing::check::check};

pub mod parse;
pub mod typing;

fn main() {
    let src = "
            (\\x : num, y : num, z : num -> x)()
        ".to_string();
    let filename = "?".to_string();

    let (ts, errs) = lexer().parse(&src).into_output_errors();

    let parse_errs = if let Some(tokens) = &ts {
        let (ast, parse_errs) = exprs_parser()
            .map_with_span(|ast, span| (ast, span))
            .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
            .into_output_errors();

        if let Some(ast) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            match check(ast.0) {
                Ok(tast) => println!("{:?}", tast),
                Err(ty_err) => {
                    let mut r = Report::build(ReportKind::Error, filename.clone(), ty_err.loc.start)
                        .with_message(ty_err.msg)
                        .with_label(Label::new((filename.clone(), ty_err.loc.into_range()))
                        .with_message(match ty_err.note {
                            Some(note) => note,
                            None => "while type checking this expression".to_string(),
                        })
                        .with_color(Color::Red)
                    );

                    if let Some((hint, loc)) = ty_err.hint {
                        r = r.with_label(Label::new((filename.clone(), loc.into_range()))
                            .with_message(hint)
                            .with_color(Color::Yellow),
                        );
                    }

                    r.finish()
                        .print(sources([(
                            filename.clone(),
                            src.clone(),
                        )]))
                        .unwrap();
                }
            }
        }

        parse_errs
    } else {
        Vec::new()
    };

    errs.into_iter()
        .map(|e| e.map_token(|c| c.to_string()))
        .chain(
            parse_errs
                .into_iter()
                .map(|e| e.map_token(|tok| tok.to_string())),
        )
        .for_each(|e| {
            Report::build(ReportKind::Error, filename.clone(), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((filename.clone(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                // .with_labels(e.contexts().map(|(label, span)| {
                //     Label::new((filename.clone(), span.into_range()))
                //         .with_message(format!("while parsing this {}", label))
                //         .with_color(Color::Yellow)
                // }))
                .finish()
                .print(sources([(filename.clone(), src.clone())]))
                .unwrap()
        });
}