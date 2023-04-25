use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
enum IRExpr<'src> {
    Int(i64),
    Var(&'src str),
    Call(&'src str, Vec<Self>),
}

impl Display for IRExpr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            IRExpr::Int(x) => write!(f, "{x}"),
            IRExpr::Var(x) => write!(f, "{x}"),
            IRExpr::Call(name, args) => {
                write!(f, "{name}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, Clone)]
enum IR<'src> {
    Define {
        name: &'src str,
        value: Box<IRExpr<'src>>,
    },
    IRExpr(IRExpr<'src>),
    Block {
        id: usize,
        body: Vec<Self>,
    },
    Func {
        name: &'src str,
        args: Vec<&'src str>,
        body: Vec<Self>,
    },
}

fn display_ir(ir: &IR, indent: usize) -> String {
    let mut s = String::new();
    for _ in 0..indent { s.push(' '); }
    match ir {
        IR::Define { name, value } => s.push_str(&format!("{name} = {value}")),
        IR::IRExpr(expr) => s.push_str(&format!("{expr}")),
        IR::Block { id, body } => {
            s.push_str(&format!("{id}:\n"));
            for ir in body {
                s.push_str(&display_ir(ir, indent + 4));
                s.push_str("\n");
            }
        },
        IR::Func { name, args, body } => {
            s.push_str(&format!("{name} "));
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { s.push_str(" "); }
                s.push_str(&format!("{arg}"));
            }
            s.push_str(":\n");
            for ir in body {
                s.push_str(&display_ir(ir, indent + 4));
                s.push_str("\n");
            }
        }
    }
    s
}

impl Display for IR<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", display_ir(self, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IR::*,
        IRExpr::*
    };

    #[test]
    fn test_ir() {
        let fns = [
            Func {
                name: "my_add",
                args: vec!["a", "b"],
                body: vec![
                    Block {
                        id: 0,
                        body: vec![
                            Define {
                                name: "v0",
                                value: Call(
                                    "add",
                                    vec![
                                        Var("a"),
                                        Var("b"),
                                    ]
                                ).into(),
                            }
                        ]
                    },
                ]
            },
            Func {
                name: "factorial",
                args: vec!["n"],
                body: vec![
                    Block {
                        id: 0,
                        body: vec![
                            Define {
                                name: "v0",
                                value: Call(
                                    "eq",
                                    vec![
                                        Var("n"),
                                        Int(1),
                                    ]
                                ).into(),
                            },
                            IRExpr(Call(
                                "jf",
                                vec![
                                    Var("v0"),
                                    Int(1),
                                ]
                            )),
                            IRExpr(Call(
                                "ret",
                                vec![
                                    Var("n"),
                                ]
                            )),
                        ]
                    },
                    Block {
                        id: 1,
                        body: vec![
                            Define {
                                name: "v0",
                                value: Call(
                                    "isub",
                                    vec![
                                        Var("n"),
                                        Int(1),
                                    ]
                                ).into(),
                            },
                            Define {
                                name: "v1",
                                value: Call(
                                    "call",
                                    vec![
                                        Var("factorial"),
                                        Var("v0"),
                                    ]
                                ).into(),
                            },
                            Define {
                                name: "v2",
                                value: Call(
                                    "imul",
                                    vec![
                                        Var("n"),
                                        Var("v1"),
                                    ]
                                ).into(),
                            },
                            IRExpr(Call(
                                "ret",
                                vec![
                                    Var("v2"),
                                ]
                            )),
                        ]
                    },
                ]
            }
        ];

        fns.iter().for_each(|ir| println!("{}", ir));
    }
}