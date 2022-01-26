use crate::compiler::instr::*;

pub fn parse(src: &str) -> Vec<Instr> {
    let mut result = Vec::new();

    for line in src.lines() {
        // <label>: <instr> <arg>+
        let mut parts = line.split_whitespace();

        let label = parts.next();
        if label == Some(";") { continue; }
        let instr = parts.next();
        
        let mut args = Vec::new();
        let mut in_quote = false;
        let mut str = String::new();
        while let Some(part) = parts.next() { match in_quote {
                true => {
                    if part.ends_with("\"") {
                        str.push_str(&format!(" {}", part));
                        args.push(str);
                        str = String::new();
                        in_quote = false;
                    } else { str.push_str(&format!(" {}", part)); }
                },
                false => {
                    if part.starts_with("$\"") {
                        str.push_str(&part);
                        in_quote = true;
                    } else { args.push(part.to_string()); }
                }
            }
        }

        result.push(match instr {
            Some("STORE") => {
                let address = args[0].parse::<Register>().unwrap();
                let value = args[1].parse::<Type>().unwrap();
                let label = label.map(|l| l[..1].parse::<usize>().unwrap()).unwrap();
                Instr::Store { address, value, label }
            },
            Some("CALL") => {
                let address = args[0].parse::<Register>().unwrap();
                let args = args[1].parse::<Register>().unwrap();
                let label = label.map(|l| l[..1].parse::<usize>().unwrap()).unwrap();
                Instr::Call { address, args, label }
            },
            Some("RETURN") => {
                let value = args[0].parse::<Register>().unwrap();
                let label = label.map(|l| l[..1].parse::<usize>().unwrap()).unwrap();
                Instr::Return { value, label }
            },
            _ => panic!("Unknown instruction: {}", instr.unwrap())
        });
    }

    result
}