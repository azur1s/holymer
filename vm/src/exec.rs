use crate::model::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Executor {
    pub stack: Vec<Value>,
    pub env: Rc<RefCell<Env>>,
    pub outer_env: Option<Rc<RefCell<Env>>>,
    pub instrs: Vec<Instr>,
    pub ip: usize,
}

#[derive(Debug)]
pub struct Error(String, usize);

impl Error {
    pub fn make<S: Into<String>>(msg: S, ip: usize) -> Self {
        Self(msg.into(), ip)
    }
}

impl Executor {
    pub fn new(instrs: Vec<Instr>) -> Self {
        Self {
            stack: Vec::new(),
            env: Rc::new(RefCell::new(Env::new())),
            outer_env: None,
            instrs,
            ip: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while self.ip < self.instrs.len() {
            self.step()?;
            self.ip += 1;
        }
        Ok(())
    }

    pub fn run_with<F: Fn(&mut Self) -> Result<(), Error>>(&mut self, f: F) -> Result<(), Error> {
        while self.ip < self.instrs.len() {
            self.step()?;
            self.ip += 1;
            f(self)?;
        }
        Ok(())
    }

    fn err(&self, msg: &str) -> Error {
        Error::make(msg, self.ip)
    }

    fn push(&mut self, v: Value) -> Result<(), Error> {
        self.stack.push(v);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.stack.pop().ok_or_else(|| self.err("stack underflow"))
    }

    fn get(&self, name: &str) -> Result<Value, Error> {
        // Get from the current environment first
        self.env
            .borrow()
            .binds
            .get(name)
            .cloned()
            // If it doesn't exist then try the outer environment
            .or_else(|| {
                self.outer_env
                    .as_ref()
                    .and_then(|env| env.borrow().binds.get(name).cloned())
                    .or(None)
            })
            .ok_or_else(|| self.err(format!("undefined variable {}", name).as_str()))
    }

    fn set(&mut self, name: &str, v: Value) -> Result<(), Error> {
        // Set the variable in the current environment if it is defined
        if self.env.borrow().binds.contains_key(name) {
            self.env.borrow_mut().binds.insert(name.to_string(), v);
        // If it is not defined in the current environment then try the outer environment
        } else if let Some(env) = &self.outer_env {
            if env.borrow().binds.contains_key(name) {
                env.borrow_mut().binds.insert(name.to_string(), v);
            } else {
                // If not then define it in the current environment
                self.env.borrow_mut().binds.insert(name.to_string(), v);
            }
        } else {
            self.env.borrow_mut().binds.insert(name.to_string(), v);
        }
        Ok(())
    }

    fn step(&mut self) -> Result<(), Error> {
        let instr = self.instrs.clone(); // TODO: maybe don't clone here
        let instr = instr
            .get(self.ip)
            .ok_or_else(|| self.err("invalid instruction pointer"))?;

        macro_rules! impl_num_binop {
            ($op:tt, $ret:ident) => {
                match (self.pop()?, self.pop()?) {
                    (Value::Num(a), Value::Num(b)) => {
                        self.stack.push(Value::$ret(a $op b));
                    }
                    _ => return Err(Error::make("can't apply operator to non-numbers", self.ip)),
                }
            };
        }
        macro_rules! impl_bool_binop {
            ($op:tt) => {
                match (self.pop()?, self.pop()?) {
                    (Value::Bool(a), Value::Bool(b)) => {
                        self.stack.push(Value::Bool(a $op b));
                    }
                    _ => return Err(Error::make("can't apply operator to non-booleans", self.ip)),
                }
            };
        }

        match instr {
            Instr::NumPush(x) => {
                self.push(Value::Num(*x))?;
            }
            Instr::NumAdd => impl_num_binop!(+, Num),
            Instr::NumSub => impl_num_binop!(-, Num),
            Instr::NumMul => impl_num_binop!(*, Num),
            Instr::NumDiv => impl_num_binop!(/, Num),
            Instr::NumMod => impl_num_binop!(%, Num),
            Instr::NumEq => impl_num_binop!(==, Bool),

            Instr::BoolPush(x) => {
                self.push(Value::Bool(*x))?;
            }
            Instr::BoolAnd => impl_bool_binop!(&&),
            Instr::BoolOr => impl_bool_binop!(||),
            Instr::BoolNot => {
                if let Value::Bool(b) = self.pop()? {
                    self.push(Value::Bool(!b))?;
                } else {
                    return Err(Error::make("can't apply `not` to non-boolean", self.ip));
                }
            }

            Instr::StrPush(x) => {
                self.push(Value::Str(x.clone()))?;
            }

            Instr::Pop => {
                self.pop()?;
            }
            Instr::Dup => {
                let v = self.pop()?;
                self.push(v.clone())?;
                self.push(v)?;
            }

            Instr::ListMake(len) => {
                let mut list = Vec::new();
                for _ in 0..*len {
                    list.push(
                        self.pop()
                            .map_err(|_| self.err("not enough arguments to make List"))?,
                    );
                }
                list.reverse();
                self.push(Value::List(list))?;
            }
            Instr::ListGet(index) => {
                if let Value::List(list) = self.pop()? {
                    let v = list
                        .get(*index)
                        .cloned()
                        .ok_or_else(|| self.err("index out of bounds"))?;
                    self.push(v)?;
                } else {
                    return Err(Error::make("can't get from non-List", self.ip));
                }
            }
            Instr::ListSet(index) => {
                if let Value::List(mut list) = self.pop()? {
                    let v = self.pop()?;
                    list.get_mut(*index)
                        .ok_or_else(|| self.err("index out of bounds"))?
                        .clone_from(&v);
                    self.push(Value::List(list))?;
                } else {
                    return Err(Error::make("can't set in non-List", self.ip));
                }
            }
            Instr::ListLen => {
                if let Value::List(list) = self.pop()? {
                    self.push(Value::Num(list.len() as i64))?;
                } else {
                    return Err(Error::make("can't get length of non-List", self.ip));
                }
            }
            Instr::ListJoin => {
                if let (Value::List(mut list1), Value::List(list2)) = (self.pop()?, self.pop()?) {
                    list1.extend(list2);
                    self.push(Value::List(list1))?;
                } else {
                    return Err(Error::make("can't join non-Lists", self.ip));
                }
            }

            Instr::FuncMake(args, instrs) => {
                let closure = Func::new(args.to_vec(), Rc::clone(&self.env), instrs.clone());
                self.push(Value::Func(closure))?;
            }
            Instr::FuncApply => {
                let v = self.pop()?;
                if let Value::Func(closure) = v {
                    // Pop the arguments
                    let mut args = Vec::new();
                    for _ in 0..closure.args.len() {
                        args.push(
                            self.pop()
                                .map_err(|_| self.err("not enough arguments to apply Function"))?,
                        );
                    }
                    args.reverse();

                    self.stack.append(&mut closure.run(args)?);
                } else {
                    return Err(Error::make(
                        format!("can't apply non-Function, got {:?}", v),
                        self.ip,
                    ));
                }
            }
            Instr::FuncCall(name) => {
                if let Value::Func(closure) = self.get(name)? {
                    let mut args = Vec::new();
                    for _ in 0..closure.args.len() {
                        args.push(
                            self.pop()
                                .map_err(|_| self.err("not enough arguments to call Function"))?,
                        );
                    }
                    args.reverse();

                    self.stack.append(&mut closure.run(args)?);
                } else {
                    return Err(Error::make("can't call non-Function", self.ip));
                }
            }

            Instr::Get(name) => {
                let v = self.get(name)?;
                self.push(v)?;
            }
            Instr::Set(name) => {
                let v = self.pop()?;
                self.set(name, v)?;
            }

            Instr::Jump(n) => {
                self.ip += n;
            }
            Instr::JumpIfFalse(n) => {
                if let Value::Bool(b) = self.pop()? {
                    if !b {
                        self.ip += n;
                    }
                } else {
                    return Err(Error::make("can't apply `if` to non-boolean", self.ip));
                }
            }

            Instr::Print => {
                let v = self.pop()?;
                println!("{}", v);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exec_expect(executor: &mut Executor, expected: Vec<Value>) {
        match executor.run() {
            Ok(_) => {
                assert_eq!(executor.stack, expected);
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_sanity() {
        let mut executor = Executor::new(vec![Instr::NumPush(1), Instr::NumPush(2), Instr::NumAdd]);
        exec_expect(&mut executor, vec![Value::Num(3)]);
    }

    #[test]
    #[should_panic]
    fn test_pop_underflow() {
        let mut executor = Executor::new(vec![Instr::NumAdd]);
        executor.run().unwrap();
    }

    #[test]
    fn test_closure() {
        let mut executor = Executor::new(vec![
            Instr::FuncMake(
                vec![],
                vec![
                    Instr::NumPush(0),
                    Instr::Set("total".to_string()),
                    Instr::FuncMake(
                        vec![],
                        vec![
                            Instr::Get("total".to_string()),
                            Instr::NumPush(1),
                            Instr::NumAdd,
                            Instr::Set("total".to_string()),
                            Instr::Get("total".to_string()),
                        ],
                    ),
                    Instr::Set("counter".to_string()),
                    Instr::Get("counter".to_string()),
                ],
            ),
            Instr::FuncApply,
            Instr::Set("tally".to_string()),
            Instr::Get("tally".to_string()),
            Instr::FuncApply,
            Instr::Get("tally".to_string()),
            Instr::FuncApply,
            Instr::Get("tally".to_string()),
            Instr::FuncApply,
        ]);
        exec_expect(
            &mut executor,
            vec![Value::Num(1), Value::Num(2), Value::Num(3)],
        );
    }
}
