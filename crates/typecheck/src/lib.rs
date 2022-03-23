use hir::{IR, IRKind};

#[derive(Debug)]
pub enum TypecheckErrorKind {
    DefinitionTypeMismatch {
        type_specified: String,
        type_found: String,
    },
}

impl std::fmt::Display for TypecheckErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TypecheckErrorKind::DefinitionTypeMismatch {
                type_specified,
                type_found,
            } => write!(
                f,
                "expected type `{}`, found type `{}`",
                type_specified, type_found
            ),
        }
    }
}

#[derive(Debug)]
pub struct TypecheckError {
    pub kind: TypecheckErrorKind,
    pub span: std::ops::Range<usize>,
}

pub fn check(ir: &Vec<IR>) -> Result<(), TypecheckError> {
    todo!();
}