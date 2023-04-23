use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::{Parser, prelude::Input};
use syntax::parser::{lexer, exprs_parser};
use typing::infer::infer_exprs;

pub mod args;

fn main() {
    let args = args::get_args();
    let filename = args.file.clone();
    let src = std::fs::read_to_string(&args.file).expect("file not found");

    let (ts, errs) = lexer().parse(&src).into_output_errors();

    let (ast, parse_errs) = if let Some(tokens) = &ts {
        let (ast, parse_errs) = exprs_parser()
            .map_with_span(|ast, span| (ast, span))
            .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
            .into_output_errors();

        (ast, parse_errs)
    } else {
        (None, vec![])
    };

    let (_typed_ast, _type_errs) = if let Some(ast) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
        let (ast, e) = infer_exprs(ast.0);
        if !e.is_empty() {
            e.iter().for_each(|e| println!("{e:?}"));
        }
        if !ast.is_empty() {
            ast.iter().for_each(|(e, _)| println!("{e:?}"));
        }
        (Some(ast), e)
    } else {
        (None, vec![])
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