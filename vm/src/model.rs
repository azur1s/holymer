use crate::exec::{Error, Executor};
use fnv::FnvHashMap;
use std::{
    cell::{Cell, RefCell},
    fmt::{Debug, Display},
    rc::Rc,
};

#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    Num(i64),
    Bool(bool),
    Str(String),
    List(Vec<Self>),
    Func(Func),
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Num(n) => write!(f, "Num({})", n),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Str(s) => write!(f, "Str({})", s),
            Value::List(xs) => write!(f, "List({:?})", xs),
            Value::Func(c) => write!(f, "Func({})", c.args.len()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::List(xs) => write!(
                f,
                "[{}]",
                xs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Func(_) => write!(f, "<Func>"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Env {
    pub binds: FnvHashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            binds: FnvHashMap::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Func {
    pub args: Vec<String>,
    pub env: Rc<RefCell<Env>>,
    pub instrs: Vec<Instr>,
}

impl Func {
    pub fn new(args: Vec<String>, env: Rc<RefCell<Env>>, instrs: Vec<Instr>) -> Self {
        Self { args, env, instrs }
    }

    pub fn run(self, args: Vec<Value>) -> Result<Vec<Value>, Error> {
        // Create a new environment for the closure
        let mut new_env = Env::new();
        for (arg, val) in self.args.iter().zip(args) {
            new_env.binds.insert(arg.clone(), val);
        }
        let new_env = Rc::new(RefCell::new(new_env));

        // Execute the closure
        let mut new_executor = Executor {
            stack: Vec::new(),
            env: new_env,
            outer_env: Some(Rc::clone(&self.env)),
            instrs: self.instrs,
            ip: 0,
        };
        new_executor.run()?;
        Ok(new_executor.stack)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instr {
    // Example: NumPush -34, NumPush 103, NumAdd
    // 00 de ff ff ff ff ff ff ff
    // 00 67 00 00 00 00 00 00 00
    // 01
    NumPush(i64), // 9 bytes: 1 byte for the enum, 8 bytes for the i64
    NumAdd,       // ┐ 1 byte
    NumSub,       // │
    NumMul,       // │
    NumDiv,       // │
    NumMod,       // │
    NumEq,        // │
    NumNe,        // │
    NumLt,        // │
    NumGt,        // │
    NumLe,        // │
    NumGe,        // ┘

    BoolPush(bool), // 2 bytes: 1 byte for the enum, 1 byte for the bool
    BoolAnd,        // ┐ 1 byte
    BoolOr,         // │
    BoolNot,        // ┘

    // StrPush:
    // ┌─┬───╶╶╶┐
    // │x│ s... [00]
    // └─┴───╶╶╶┘
    // where x is the enum (1 byte)
    //       s is the string (n bytes)
    // Example: StrPush "Hello, World!"
    // [05] [48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21] [00]
    // └────┼────────────────────────────────────────┼─╼ enum
    //      └────────────────────────────────────────┼─╼ string
    //                                               └─╼ null delimiter
    // Total of 15 bytes (1 + 13 + 1)
    StrPush(String), // 1 + string.len() + 1 bytes
    StrConcat,       // 1 byte

    Pop, // ┐ 1 byte
    Dup, // ┘

    ListMake(usize), // ┐ 9 bytes: 1 byte for the enum, 8 bytes for the usize (64-bit)
    ListGet(usize),  // │
    ListSet(usize),  // ┘
    ListLen,         // ┐ 1 byte
    ListJoin,        // ┘

    // FuncMake:
    // ┌─┬───┬───┬─────╶╶╶┬──────╶╶╶╶╶
    // │x│ n │ m │  a...  │ i...
    // └─┴───┴───┴─────╶╶╶┴──────╶╶╶╶╶
    // where x is the enum (1 byte)
    //       n is the number of arguments (8 bytes)
    //       m is the number of instructions (8 bytes)
    //       a is the arguments (n bytes, null delimited)
    //       ╴╴┬──────┬────┬╶╶
    //         │ s... │ 00 │ // For example: "a", "bc" -> [61 00 62 63 00]
    //       ╴╴┴──────┴────┴╶╶
    //       i is the instructions (m bytes)
    // Example: FuncMake ["x", "y"] [Get "x", Get "yz", NumAdd]
    // [0d] [02 ..] [03 ..] [78 00 79 7a 00] [16 78 00 16 79 7a 00 01]
    // └────┼───────┼───────┼────────────────┼─╼ enum
    //      └───────┼───────┼────────────────┼─╼ number of arguments
    //              └───────┼────────────────┼─╼ number of instructions
    //                      └────────────────┼─╼ arguments (null delimited)
    //                                       └─╼ instructions
    FuncMake(Vec<String>, Vec<Instr>), // 1 + 8 + 8 + args.len() + instrs.len() bytes
    FuncApply,                         // 1 byte
    FuncCall(String),                  // 1 + string.len() + 1 bytes

    Get(String), // ┐ 1 + string.len() + 1 bytes
    Set(String), // ┘

    Jump(usize),        // ┐ 9 bytes: 1 byte for the enum, 8 bytes for the usize (64-bit)
    JumpIfFalse(usize), // ┘

    Print,   // ┐ 1 byte
    PrintLn, // ┘
}

static mut INSTR_INDEX: Cell<u8> = Cell::new(0);

impl Instr {
    pub fn size(&self) -> usize {
        match self {
            Instr::NumPush(_) => 1 + std::mem::size_of::<i64>(),
            Instr::NumAdd
            | Instr::NumSub
            | Instr::NumMul
            | Instr::NumDiv
            | Instr::NumMod
            | Instr::NumEq
            | Instr::NumNe
            | Instr::NumLt
            | Instr::NumGt
            | Instr::NumLe
            | Instr::NumGe => 1,

            Instr::BoolPush(_) => 1 + std::mem::size_of::<bool>(),
            Instr::BoolAnd | Instr::BoolOr | Instr::BoolNot => 1,

            Instr::StrPush(s) => 1 + s.len() + 1,
            Instr::StrConcat => 1,

            Instr::Pop | Instr::Dup => 1,

            Instr::ListMake(_) | Instr::ListGet(_) | Instr::ListSet(_) => {
                1 + std::mem::size_of::<usize>()
            }
            Instr::ListLen | Instr::ListJoin => 1,

            Instr::FuncMake(args, instrs) => {
                1 + 8
                    + 8
                    + args.iter().map(|s| s.len() + 1).sum::<usize>()
                    + instrs.iter().map(|i| i.size()).sum::<usize>()
            }
            Instr::FuncApply => 1,
            Instr::FuncCall(s) => 1 + s.len() + 1,

            Instr::Get(s) | Instr::Set(s) => 1 + s.len() + 1,

            Instr::Jump(_) | Instr::JumpIfFalse(_) => 1 + std::mem::size_of::<usize>(),

            Instr::Print | Instr::PrintLn => 1,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // A macro that will return the next index and increment it
        // so we don't have to rewrite all the first bytes again when
        // we changes the order or add new instructions
        macro_rules! index {
            () => {
                unsafe {
                    let i = INSTR_INDEX.get();
                    INSTR_INDEX.set(i + 1);
                    i
                }
            };
        }

        let mut bytes = Vec::new();
        match self {
            Instr::NumPush(n) => {
                bytes.push(index!());
                bytes.extend(n.to_le_bytes());
            }
            Instr::NumAdd
            | Instr::NumSub
            | Instr::NumMul
            | Instr::NumDiv
            | Instr::NumMod
            | Instr::NumEq
            | Instr::NumNe
            | Instr::NumLt
            | Instr::NumGt
            | Instr::NumLe
            | Instr::NumGe => bytes.push(index!()),

            Instr::BoolPush(b) => {
                bytes.push(index!());
                bytes.push(*b as u8);
            }
            Instr::BoolAnd => bytes.push(index!()),
            Instr::BoolOr => bytes.push(index!()),
            Instr::BoolNot => bytes.push(index!()),

            Instr::StrPush(s) => {
                bytes.push(index!());
                bytes.extend(s.as_bytes());
                bytes.push(0x00);
            }
            Instr::StrConcat => bytes.push(index!()),

            Instr::Pop => bytes.push(index!()),
            Instr::Dup => bytes.push(index!()),

            Instr::ListMake(n) => {
                bytes.push(index!());
                bytes.extend(n.to_le_bytes());
            }
            Instr::ListGet(n) => {
                bytes.push(index!());
                bytes.extend(n.to_le_bytes());
            }
            Instr::ListSet(n) => {
                bytes.push(index!());
                bytes.extend(n.to_le_bytes());
            }
            Instr::ListLen => bytes.push(index!()),
            Instr::ListJoin => bytes.push(index!()),

            Instr::FuncMake(args, instrs) => {
                bytes.push(index!());
                bytes.extend((args.len() as u64).to_le_bytes());
                bytes.extend((instrs.len() as u64).to_le_bytes());
                for arg in args {
                    bytes.extend(arg.as_bytes());
                    bytes.push(0x00);
                }
                for instr in instrs {
                    bytes.extend(instr.to_bytes());
                }
            }
            Instr::FuncApply => bytes.push(index!()),
            Instr::FuncCall(s) => {
                bytes.push(index!());
                bytes.extend(s.as_bytes());
                bytes.push(0x00);
            }

            Instr::Get(s) => {
                bytes.push(index!());
                bytes.extend(s.as_bytes());
                bytes.push(0x00);
            }
            Instr::Set(s) => {
                bytes.push(index!());
                bytes.extend(s.as_bytes());
                bytes.push(0x00);
            }

            Instr::Jump(n) => {
                bytes.push(index!());
                bytes.extend(n.to_le_bytes());
            }
            Instr::JumpIfFalse(n) => {
                bytes.push(index!());
                bytes.extend(n.to_le_bytes());
            }

            Instr::Print => bytes.push(index!()),
            Instr::PrintLn => bytes.push(index!()),
        }
        bytes
    }
}
