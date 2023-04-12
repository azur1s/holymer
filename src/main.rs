use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::{Parser, prelude::Input};
use self::{parse::parser::{lexer, exprs_parser}};

pub mod parse;
pub mod typing;

fn main() {
    let src = "
            {
                let foo =
                    let a = true in
                        let b = false in
                            a + b;
                foo * 2
            }
        ".to_string();
    let filename = "?".to_string();

    let (ts, errs) = lexer().parse(&src).into_output_errors();

    let parse_errs = if let Some(tokens) = &ts {
        let (ast, parse_errs) = exprs_parser()
            .map_with_span(|ast, span| (ast, span))
            .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
            .into_output_errors();

        if let Some(ast) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            println!("{:?}", ast);
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