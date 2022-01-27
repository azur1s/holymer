use regex::Regex;

use crate::vm::instr::*;

const REGEX: &str = r###"[^\s\$";]+|\$"[^"]*"|;.*"###;

macro_rules! value    { ($s:expr) => { $s.parse::<Type>().unwrap() }; }
macro_rules! register { ($s:expr) => { $s.parse::<Register>().unwrap() }; }
macro_rules! label    { ($s:expr) => { $s.parse::<usize>().unwrap() }; }

pub fn parse_instr(src: &str) -> Vec<Instr> {
    let regex = Regex::new(REGEX).unwrap();
    let mut result = Vec::new();

    for line in src.lines() {
        let tokens = regex.find_iter(line).map(|m| m.as_str()).collect::<Vec<_>>();
        if tokens[0].starts_with(";") { continue; }

        let label = label!(tokens[0]);

        match tokens[1] {
            "STORE" => { result.push(Instr::Store {
                address: register!(tokens[2]),
                value: value!(tokens[3]),
                label,
            }); },
            "CALL" => { result.push(Instr::Call {
                address: register!(tokens[2]),
                args: register!(tokens[3]),
                label,
            }); },
            "PUSH" => { result.push(Instr::Push {
                value: value!(tokens[2]),
                label,
            }); },
            "POP" => { result.push(Instr::Pop {
                address: register!(tokens[2]),
                label,
            }); },
            "ADD" => { result.push(Instr::Add { label }); },
            "SUB" => { result.push(Instr::Sub { label }); },
            "MUL" => { result.push(Instr::Mul { label }); },
            "DIV" => { result.push(Instr::Div { label }); },
            "JMP" => { result.push(Instr::Jump {
                to: label!(tokens[2]),
                label,
            }); },
            "POP_JUMP_IF_FALSE" => { result.push(Instr::PopJumpIfFalse {
                to: label!(tokens[2]),
                label,
            }); },
            "RETURN" => { result.push(Instr::Return {
                value: register!(tokens[2]),
                label,
            }); },
            _ => panic!("Unknown instruction: {}", tokens[1]),
        }
    }

    result
}