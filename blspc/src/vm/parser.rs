use regex::Regex;

use crate::vm::{instr::*, types::Type};

const REGEX: &str = r###"\((?:[^)(]+|\((?:[^)(]+|\([^)(]*\))*\))*\)|[^\s\$";]+|"[^"]*"|;.*"###;

macro_rules! value    { ($s:expr) => { $s.parse::<Type>().unwrap() }; }
macro_rules! register { ($s:expr) => { $s.parse::<Register>().unwrap() }; }

pub fn parse_instr(src: &str) -> Vec<Instr> {
    let regex = Regex::new(REGEX).unwrap();
    let mut result = Vec::new();

    for line in src.lines() {
        let tokens = regex.find_iter(line).map(|m| m.as_str()).collect::<Vec<_>>();
        if tokens[0].starts_with(";") { continue; }

        match tokens[0] {
            "LOAD"  => { result.push(Instr::Load { address: register!(tokens[1].to_string()) }); },
            "STORE" => { result.push(Instr::Store { address: register!(tokens[1].to_string()) }); },

            "CALL"  => { result.push(Instr::Call { function: tokens[1].to_string() }); },

            "PUSH"  => { result.push(Instr::Push { value: value!(tokens[1]) }); },
            "POP"   => { result.push(Instr::Pop { address: register!(tokens[1]) }); },
            "SWAP"  => { result.push(Instr::Swap); },

            "ADD"   => { result.push(Instr::Add); },
            "SUB"   => { result.push(Instr::Sub); },
            "MUL"   => { result.push(Instr::Mul); },
            "DIV"   => { result.push(Instr::Div); },

            "NOT"   => { result.push(Instr::Not); },

            "JMPL"  => { result.push(Instr::JumpLabel { to: tokens[1].to_string() }); },
            "JMP"   => { result.push(Instr::Jump { to: tokens[1].parse::<isize>().unwrap() }); },
            "JMPF"  => { result.push(Instr::JumpIfFalse { to: tokens[1].parse::<isize>().unwrap() }); },

            "EQ"    => { result.push(Instr::Equal); },

            "RET"   => { result.push(Instr::Return); },
            _ => {
                if tokens[0].starts_with(".") {
                    let name = &tokens[0][1..tokens[0].len() - 1];
                    result.push(Instr::Label { name: name.to_string() });
                }
            },
        }
    }

    result
}