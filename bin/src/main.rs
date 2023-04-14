use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::{Parser, prelude::Input};
use syntax::parser::{lexer, exprs_parser};
use typing::infer::infer_exprs;

fn main() {
    let src = "
            let r = {
                let x =
                    if 0 == 1
                    then {
                        let x = true;
                        if x then 1 else 2
                    }
                    else 34 + {
                        let foo = 30 in
                            foo + 5
                    };
                let y = { 1 } * 2;
                if 1 + 1 == 2
                then x
                else y
            };
        ".to_string();
    let filename = "?".to_string();

    let (ts, errs) = lexer().parse(&src).into_output_errors();

    let parse_errs = if let Some(tokens) = &ts {
        let (ast, parse_errs) = exprs_parser()
            .map_with_span(|ast, span| (ast, span))
            .parse(tokens.as_slice().spanned((src.len()..src.len()).into()))
            .into_output_errors();

        if let Some(ast) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            let (ast, e) = infer_exprs(ast.0);
            if !e.is_empty() {
                println!("{:?}", e);
            }
            if !ast.is_empty() {
                println!("{:?}", ast);
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