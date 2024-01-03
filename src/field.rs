use std::{fmt::Display, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FieldKind {
    RealScalar,
    ComplexScalar(bool),
    RealVector,
    ComplexVector(bool),
    Spinor(bool),
}

pub use FieldKind::*;

impl FieldKind {
    pub fn anti(&self) -> Self {
        match self {
            RealScalar => RealScalar,
            ComplexScalar(t) => ComplexScalar(!t),
            RealVector => RealVector,
            ComplexVector(t) => ComplexVector(!t),
            Spinor(t) => Spinor(!t),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Field {
    kind: FieldKind,
    name: Rc<str>,
}

impl Field {
    pub fn new(kind: FieldKind, name: &str) -> Self {
        Field {
            kind,
            name: Rc::from(name),
        }
    }

    pub fn kind(&self) -> FieldKind {
        self.kind
    }

    pub fn anti(&self) -> Self {
        Self {
            kind: self.kind.anti(),
            name: Rc::clone(&self.name),
        }
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ComplexScalar(t) | ComplexVector(t) | Spinor(t) => {
                if t {
                    write!(f, "\\bar ")?;
                }
            }
            _ => (),
        }
        write!(f, "{}", self.name)
    }
}
