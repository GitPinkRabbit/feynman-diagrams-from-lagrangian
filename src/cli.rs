use clap::Parser;
use std::collections::VecDeque;

use crate::{field::*, lagrangian::*};

/// Produce Feynman diagrams from Lagrangian
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Lagrangian, without coefficients
    #[arg(short, long)]
    lagrangian: Option<String>,

    /// Maximum order of the diagram
    #[arg(short)]
    n: u32,
}

pub fn parse() -> (CheckedLagrangian, u32) {
    let cli = Cli::parse();

    let lag;

    if let Some(x) = cli.lagrangian {
        lag = x;
    } else if true {
        lag = String::from(r"\phi\bar\psi\psi");
    } else {
        panic!("必须提供拉氏量");
    }

    let mut lag: VecDeque<_> = lag.trim().chars().collect();
    lag.push_front('+');

    let res = || -> Result<UncheckedLagrangian, ()> {
        let mut res = UncheckedLagrangian::new();
        let drop_whitespace = |seq: &mut VecDeque<char>| {
            while let Some(c) = seq.front() {
                if !c.is_whitespace() {
                    break;
                }
                seq.pop_front();
            }
        };
        loop {
            drop_whitespace(&mut lag);
            match lag.pop_front() {
                None => break,
                Some('%') => break,
                Some('+') => (),
                Some(_) => return Err(()),
            };
            let mut factors = vec![];
            loop {
                let get_token = |seq: &mut VecDeque<char>| -> Result<Option<String>, ()> {
                    drop_whitespace(seq);
                    match seq.pop_front() {
                        None => Ok(None),
                        Some('{') | Some('}') => Err(()),
                        Some('+') => {
                            seq.push_front('+');
                            Ok(None)
                        }
                        Some('\\') => {
                            if seq.front().is_some() {
                                let mut res = String::from('\\');
                                let c = seq.front().unwrap();
                                if c.is_ascii_alphabetic() {
                                    while let Some(c) = seq.front() {
                                        if !c.is_ascii_alphabetic() {
                                            break;
                                        }
                                        res.push(*c);
                                        seq.pop_front();
                                    }
                                } else {
                                    res.push(*c);
                                    seq.pop_front();
                                }
                                Ok(Some(res))
                            } else {
                                Err(())
                            }
                        }
                        Some(c) => Ok(Some(String::from(c))),
                    }
                };
                let token = if let Some(t) = get_token(&mut lag)? {
                    t
                } else {
                    break;
                };
                let (token, bar) = if token == "\\bar" {
                    if let Some(t) = get_token(&mut lag)? {
                        (t, true)
                    } else {
                        return Err(());
                    }
                } else {
                    (token, false)
                };

                let get_block = |seq: &mut VecDeque<char>| -> Result<String, ()> {
                    drop_whitespace(seq);
                    Ok(match seq.pop_front() {
                        None | Some('}') => return Err(()),
                        Some('\\') => {
                            if seq.front().is_some() {
                                let mut res = String::from('\\');
                                let c = seq.front().unwrap();
                                if c.is_ascii_alphabetic() {
                                    while let Some(c) = seq.front() {
                                        if !c.is_ascii_alphabetic() {
                                            break;
                                        }
                                        res.push(*c);
                                        seq.pop_front();
                                    }
                                } else {
                                    res.push(*c);
                                    seq.pop_front();
                                }
                                res
                            } else {
                                return Err(());
                            }
                        }
                        Some('{') => {
                            let mut res = String::from('{');
                            let mut layer = 1;
                            while let Some(c) = seq.pop_front() {
                                match c {
                                    '{' => layer += 1,
                                    '}' => layer -= 1,
                                    _ => (),
                                };
                                res.push(c);
                                if layer == 0 {
                                    break;
                                }
                            }
                            if layer != 0 {
                                return Err(());
                            }
                            res
                        }
                        Some(c) => String::from(c),
                    })
                };
                drop_whitespace(&mut lag);
                let subscript = if lag.front().is_some_and(|c| c == &'_') {
                    lag.pop_front();
                    Some(get_block(&mut lag)?)
                } else {
                    None
                };
                drop_whitespace(&mut lag);
                let mut superscript = if lag.front().is_some_and(|c| c == &'^') {
                    lag.pop_front();
                    Some(get_block(&mut lag)?)
                } else {
                    None
                };
                let num: u8 =
                    if let Ok(v) = superscript.clone().unwrap_or(String::from("1")).parse() {
                        superscript = None;
                        v
                    } else {
                        1
                    };

                let kind = match token.as_str() {
                    "\\phi" => RealScalar,
                    "\\varphi" => ComplexScalar(bar),
                    "A" => RealVector,
                    "F" => ComplexVector(bar),
                    "\\psi" => Spinor(bar),
                    _ => return Err(()),
                };
                let mut name = token;
                if let Some(sub) = subscript {
                    name.push('_');
                    name.push_str(&sub);
                }
                if let Some(sup) = superscript {
                    name.push('^');
                    name.push_str(&sup);
                }
                factors.append(&mut vec![Field::new(kind, &name); num as usize])
            }
            res.push(Interaction::new(factors.into_iter()));
        }
        Ok(res)
    }();

    (CheckedLagrangian::new(res.expect("拉氏量格式有误")), cli.n)
}
