use crate::middle::ir::{IR, Value};

pub fn gen(irs: Vec<IR>) -> String {
    let mut output = String::new();
    for ir in irs {
        output.push_str(&gen_ir(&ir));
    }
    output
}

fn gen_ir(ir: &IR) -> String {
    match ir {
        IR::Define { name, type_hint: _, value } => { // type_hint is only used in type_checking i think
            let value = gen_ir(value);
            format!("const {} = {};", name, value)
        },
        IR::Fun { name, return_type_hint: _, args, body } => {
            let args = args
                .iter()
                .map(|(name, _)| format!("{}", name))
                .collect::<Vec<_>>()
                .join(", ");
            
            let body = match &**body {
                IR::Value { value } => gen_value(value),
                IR::Do { body } => {
                    let mut out = String::new();
                    for (i, node) in body.iter().enumerate() {
                        if i == body.len() - 1 {
                            out.push_str(format!("return {};", gen_ir(node)).as_str());
                        } else {
                            out.push_str(&gen_ir(node));
                        }
                    }
                    out
                },
                IR::Binary { op, left, right } => {
                    format!(
                        "return {} {} {};",
                        gen_ir(left),
                        op,
                        gen_ir(right)
                    )
                },
                _ => { println!("{:?}", body); todo!() }
            };

            format!(
                "const {} = ({}) => {{ {} }};",
                name,
                args,
                body
            )
        },
        IR::Call { name, args } => {
            match name.as_str() {
                "print" => {
                    let args = gen_ir(&args[0]);
                    format!("console.log({});", args.trim_end_matches(";"))
                },
                _ => {
                    let args = args
                        .iter()
                        .map(|arg| gen_ir(arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}({})", name, args)
                },
            }
        },
        IR::Value { value } => {
            gen_value(value)
        },
        IR::Binary { op, left, right } => {
            let left = gen_ir(left);
            let right = gen_ir(right);
            format!("({} {} {});", left, op, right)
        },
        _ => { println!("{:?}", ir); todo!() }
    }
}

fn gen_value(value: &Value) -> String {
    match value {
        Value::Int(i)    => format!("{}", i),
        Value::Float(f)  => format!("{}", f),
        Value::Bool(b)   => format!("{}", b),
        Value::String(s) => format!("\"{}\"", s),
        Value::Ident(s)  => format!("{}", s),
    }
}