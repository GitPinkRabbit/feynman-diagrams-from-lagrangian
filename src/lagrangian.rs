use itertools::Itertools;
use std::fmt::Display;

use crate::field::*;

#[derive(Clone, Debug)]
pub struct Interaction {
    factors: Vec<Field>,
}

impl Interaction {
    pub fn new(factors: impl Iterator<Item = Field>) -> Self {
        let factors: Vec<_> = factors.collect();
        if factors.len() < 3 {
            panic!("存在非相互作用项（传播子）");
        }
        if factors.len() > 4 {
            panic!("相互作用项的乘数不应超过四个");
        }

        let mut scalar_number = 0;
        let mut vector_number = 0;
        let mut spinor_number = 0;
        factors.iter().for_each(|f| {
            let h = |t| if t { -1 } else { 1 };
            match f.kind() {
                ComplexScalar(t) => scalar_number += h(t),
                ComplexVector(t) => vector_number += h(t),
                Spinor(t) => spinor_number += h(t),
                _ => (),
            }
        });

        if scalar_number != 0 || vector_number != 0 || spinor_number != 0 {
            panic!("相互作用项违反了守恒律");
        }

        Self { factors }
    }

    pub fn factors(&self) -> &[Field] {
        &self.factors
    }
}

impl Display for Interaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.factors.iter().join(" * "))
    }
}

#[derive(Clone, Debug)]
pub struct UncheckedLagrangian {
    fields: Vec<Field>,
    interactions: Vec<Interaction>,
}

impl UncheckedLagrangian {
    pub fn new() -> Self {
        UncheckedLagrangian {
            fields: vec![],
            interactions: vec![],
        }
    }

    pub fn push(&mut self, int: Interaction) {
        self.interactions.push(int.clone());
        for f in &int.factors {
            if !self.fields.iter().any(|x| x == f) {
                self.fields.push(f.clone());
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckedLagrangian {
    inner: UncheckedLagrangian,
}

impl CheckedLagrangian {
    pub fn new(inner: UncheckedLagrangian) -> Self {
        for f in inner.fields.iter() {
            inner
                .fields
                .iter()
                .find(|&x| x == &f.anti())
                .expect("存在未配对的场算符");
        }

        CheckedLagrangian { inner }
    }

    pub fn fields(&self) -> &[Field] {
        &self.inner.fields
    }

    pub fn interactions(&self) -> &[Interaction] {
        &self.inner.interactions
    }
}

impl Display for CheckedLagrangian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.interactions().iter().join(" + "))
    }
}
