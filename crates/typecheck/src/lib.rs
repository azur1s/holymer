use hir::{IR, IRKind, Value};

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

pub fn check(irs: &[IR]) -> Result<(), Vec<TypecheckError>> {
    let mut errors = Vec::new();
    for ir in irs {
        match &ir.kind {
            ir @ IRKind::Define { .. } => {
                match check_define(&ir) {
                    Ok(()) => (),
                    Err(e) => errors.push(e),
                }
            }
            _ => {}
        }
    }
    if errors.is_empty() { Ok(()) }
    else { Err(errors) }
}

#[macro_export]
macro_rules! return_err {
    ($kind:expr, $span:expr) => {{
        return Err(TypecheckError {
            kind: $kind,
            span: $span.clone()
        });
    }};
}

/// Check the types of the definitions.
/// This is done by checking the type of the value against the type hint.
/// 
/// # Examples
/// ```sml
/// let x: int = 1; -- Correct
/// 
/// let x: string = 1; -- Incorrect
/// ```
fn check_define(ir: &IRKind) -> Result<(), TypecheckError> {
    match ir {
        IRKind::Define {
            type_hint,
            value,
            span,
            ..
        } => {
            match &**value {
                IRKind::Value { value } => {
                    match value {
                        Value::Int(_) => {
                            if type_hint != "number" {
                                return_err!(
                                    TypecheckErrorKind::DefinitionTypeMismatch {
                                        type_specified: type_hint.to_string(),
                                        type_found: "number".to_string(),
                                    },
                                    span.clone()
                                );
                            }
                        }
                        Value::String(_) => {
                            if type_hint != "string" {
                                return_err!(
                                    TypecheckErrorKind::DefinitionTypeMismatch {
                                        type_specified: type_hint.to_string(),
                                        type_found: "string".to_string(),
                                    },
                                    span.clone()
                                );
                            }
                        }
                        Value::Boolean(_) => {
                            if type_hint != "boolean" {
                                return_err!(
                                    TypecheckErrorKind::DefinitionTypeMismatch {
                                        type_specified: type_hint.to_string(),
                                        type_found: "boolean".to_string(),
                                    },
                                    span.clone()
                                );
                            }
                        }
                        // TODO: other types
                        _ => {}
                    }
                }
                // TODO: other (right-hand side) IRKinds
                _ => {}
            }
        }
        _ => unreachable!()
    }
    Ok(())
}