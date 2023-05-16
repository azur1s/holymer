use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::{Parser, prelude::Input};

use syntax::parser::{lexer, exprs_parser};
use typing::infer::{infer_exprs, InferErrorKind};
use ir::Lowerer;

pub mod args;

fn main() {
    let args = args::get_args();
    let filename = args.file.clone();
    let src = std::fs::read_to_string(&args.file).expect("file not found");

    // Lexing & parsing
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

    // Typecheck if there are no lexing or parsing errors
    if let Some(ast) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
        let (ast, e) = infer_exprs(ast.0);
        // If there is an error, print it
        if !e.is_empty() {
            e.into_iter()
                .for_each(|e| {
                    let mut r = Report::build(ReportKind::Error, filename.clone(), e.span.start)
                        .with_message(e.title.to_string());

                    for (msg, kind, span) in e.labels {
                        r = r.with_label(
                            Label::new((filename.clone(), span.into_range()))
                                .with_message(msg.to_string())
                                .with_color(match kind {
                                    InferErrorKind::Error => Color::Red,
                                    InferErrorKind::Hint => Color::Blue,
                                }),
                        );
                    }

                    r
                        .finish()
                        .print(sources([(filename.clone(), src.clone())]))
                        .unwrap()
                });
        // Else go to the next stage
        } else {
            // ast.iter().for_each(|node| println!("{:?}", node.0));
            let mut l = Lowerer::new();
            let irs = l.lower_texprs(ast);
            irs.iter().for_each(|ir| println!("{:?}", ir));
        }
    };

    // Report lex & parse errors
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
                .finish()
                .print(sources([(filename.clone(), src.clone())]))
                .unwrap()
        });
}