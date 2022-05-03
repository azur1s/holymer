use syntax::{ast::*, lex::Span};

/// A struct that contains emitted code.
pub struct Codegen {
    /// The emitted code.
    /// When the codegen is done, this will be joined into a single string
    pub emitted: Vec<String>,
}

impl Default for Codegen { fn default() -> Self { Self::new() } }

impl Codegen {
    pub fn new() -> Codegen {
        Codegen { emitted: Vec::new() }
    }

    /// Emit a string to the output.
    pub fn emit<S: Into<String>>(&mut self, s: S) {
        self.emitted.push(s.into());
    }

    pub fn gen(&mut self, ast: Vec<(Expr, Span)>) {
        for (expr, _) in ast {
            self.emit(self.gen_expr(&expr, true));
        }
    }

    fn gen_expr(&self, expr: &Expr, semicolon: bool) -> String {
        #[macro_export]
        macro_rules! semicolon { () => { if semicolon { ";" } else { "" } }; }

        match expr {
            Expr::Literal(lit)     => self.gen_literal(&lit.0),
            Expr::Identifier(name) => { format!("_{}{}", name.0, semicolon!()) },
            Expr::Tuple(elems)     => { format!("({}{})", elems.iter().map(|e| self.gen_expr(&e.0, false)).collect::<Vec<_>>().join(", "), semicolon!()) },
            Expr::Vector(elems)    => { format!("[{}{}]", elems.iter().map(|e| self.gen_expr(&e.0, false)).collect::<Vec<_>>().join(", "), semicolon!()) },
            Expr::Object { fields } => {
                format!("{{{}}}",
                    fields.iter().map(|(name, expr)| format!("{}: {}", name.0, self.gen_expr(&expr.0, false))).collect::<Vec<_>>().join(",\n "))
            },

            Expr::Unary { op, rhs } => { format!("{}{}", op, self.gen_expr(&rhs.0, false)) },
            Expr::Binary { op, lhs, rhs } => {
                format!("{}{}{}{}", self.gen_expr(&lhs.0, false), op, self.gen_expr(&rhs.0, false), semicolon!())
            },

            Expr::Call { name, args } => {
                format!(
                    "{}({}){}",
                    self.gen_expr(&name.0, false),
                    args
                        .iter()
                        .map(|arg| self.gen_expr(&arg.0, false))
                        .collect::<Vec<_>>()
                        .join(", "),
                    semicolon!())
            },
            Expr::Method { obj, name, args } => {
                format!(
                    "{}.{}({}){}",
                    self.gen_expr(&obj.0, false),
                    self.gen_expr(&name.0, false).trim_start_matches('_'),
                    args
                        .iter()
                        .map(|arg| self.gen_expr(&arg.0, false))
                        .collect::<Vec<_>>()
                        .join(", "),
                    semicolon!())
            },
            Expr::Access { obj, name } => {
                format!("{}.{}", self.gen_expr(&obj.0, false), self.gen_expr(&name.0, false).trim_start_matches('_'))
            },
            Expr::Intrinsic { name, args } => {
                if let Expr::Identifier(name) = &name.0 {
                    match name.0.as_str() {
                        "write" => { format!("console.log({})", args.iter().map(|arg| self.gen_expr(&arg.0, false)).collect::<Vec<_>>().join(", ")) },
                        _ => unimplemented!(),
                    }
                } else {
                    panic!("Expected identifier for intrinsic name");
                }
            },

            Expr::Define { name, typehint, value } => {
                format!(
                    "let {} : {} = {}{}",
                    name.0,
                    self.gen_typehint(&typehint.0),
                    self.gen_expr(&value.0, false),
                    semicolon!())
            },
            Expr::Redefine { name, value } => {
                format!(
                    "{} = {}{}",
                    name.0,
                    self.gen_expr(&value.0, false),
                    semicolon!())
            },

            Expr::Function { name, generics, args, typehint, body } => {
                format!(
                    "const _{} = {}({}): {} => {{{}}}{}\n",
                    name.0,
                    if generics.is_empty() { "".to_string() } else {
                        format!("<{}>",
                            generics.iter().map(|g| g.0.clone()).collect::<Vec<_>>().join(", "))
                    },
                    args
                        .iter()
                        .map(|arg| format!("_{}: {}", arg.0.0, self.gen_typehint(&arg.1.0)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.gen_typehint(&typehint.0),
                    self.gen_expr(&body.0, false),
                    semicolon!())
            },

            Expr::If { cond, t, f } => {
                format!(
                    "if {} {{{}}} else {{{}}}",
                    self.gen_expr(&cond.0, false),
                    self.gen_expr(&t.0, false),
                    self.gen_expr(&f.0, false))
            },

            Expr::Do { body } => {
                format!(
                    "{{\n{}}}\n",
                    body.0.iter().map(|e| self.gen_expr(&e.0, false)).collect::<Vec<_>>().join("\n"))
            },

            Expr::Return(expr) => {
                format!("return {}\n", self.gen_expr(&expr.0, true))
            },

            #[allow(unreachable_patterns)]
            _ => { dbg!(expr); todo!() },
        }
    }

    fn gen_literal(&self, lit: &Literal) -> String {
        match lit {
            Literal::Int(i)     => format!("{}", i),
            Literal::String(s)  => format!("\"{}\"", s),
            Literal::Boolean(b) => format!("{}", b),
        }
    }

    fn gen_typehint(&self, typehint: &Typehint) -> String {
        match typehint {
            Typehint::Builtin(ty) => {
                match ty {
                    BuiltinType::Any       => "any",
                    BuiltinType::Null      => "null",
                    BuiltinType::Undefined => "undefined",
                    BuiltinType::Boolean   => "boolean",
                    BuiltinType::Int       => "number",
                    BuiltinType::String    => "string",
                }.to_string()
            },
            Typehint::Single(ty) => ty.clone(),

            Typehint::Tuple(tys) => format!("[{}]", tys
                .iter()
                .map(|ty| self.gen_typehint(&ty.0)).collect::<Vec<_>>().join(", ")),
            Typehint::Vector(ty) => format!("{}[]", self.gen_typehint(&ty.0)),

            Typehint::Function(args, ret) => {
                let args_ty = args.iter().map(|arg| self.gen_typehint(&arg.0)).collect::<Vec<_>>();
                let return_ty =  self.gen_typehint(&ret.0);
                format!( "({}) => {}",
                    args_ty
                        .iter()
                        .enumerate()
                        .map(|(i, arg)| format!("__{}: {}", i, arg)) // Maybe use this in the future
                        .collect::<Vec<_>>()
                        .join(", "),
                    return_ty)
            },

            Typehint::Union(tys) => tys
                .iter()
                .map(|ty| self.gen_typehint(&ty.0)).collect::<Vec<_>>().join(" | "),
        }
    }
}