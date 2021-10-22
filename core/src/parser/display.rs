use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::parser::literals::{Literal, StrictNumber, VagueLiteral};
use crate::parser::ops::Op;
use crate::parser::{Token, TonsOfTokens};

static INDENT: AtomicUsize = AtomicUsize::new(0);
static DO_INDENT: AtomicBool = AtomicBool::new(false);

impl fmt::Display for VagueLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &VagueLiteral::Float(i) => write!(f, "{i}vf"),
            &VagueLiteral::Integer(i) => write!(f, "{i}v"),
            &VagueLiteral::String(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for StrictNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &StrictNumber::Byte(i) => write!(f, "{i}b"),
            &StrictNumber::ByteSigned(i) => write!(f, "{i}bi"),
            &StrictNumber::Small(i) => write!(f, "{i}s"),
            &StrictNumber::SmallSigned(i) => write!(f, "{i}si"),
            &StrictNumber::Medium(i) => write!(f, "{i}m"),
            &StrictNumber::MediumSigned(i) => write!(f, "{i}mi"),
            &StrictNumber::MediumFloat(i) => write!(f, "{i}m"),
            &StrictNumber::Large(i) => write!(f, "{i}l"),
            &StrictNumber::LargeSigned(i) => write!(f, "{i}li"),
            &StrictNumber::LargeFloat(i) => write!(f, "{i}l"),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &Op::Add(t1, t2) => {
                write!(f, "({t1} + {t2})")
            }

            &Op::Subtract(t1, t2) => {
                write!(f, "({t1} - {t2})")
            }

            &Op::Eq(t1, t2) => {
                write!(f, "({t1} == {t2})")
            }

            &Op::Neq(t1, t2) => {
                write!(f, "({t1} != {t2})")
            }
            &Op::Gt(t1, t2) => {
                write!(f, "({t1} > {t2})")
            }
            &Op::Lt(t1, t2) => {
                write!(f, "({t1} < {t2})")
            }
            &Op::Gte(t1, t2) => {
                write!(f, "({t1} >= {t2})")
            }
            &Op::Lte(t1, t2) => {
                write!(f, "({t1} <= {t2})")
            }
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &Literal::Number(num) => write!(f, "{num}"),
            &Literal::Vague(num) => write!(f, "{num}"),
            &Literal::Bool(b) => write!(f, "{b}"),
            &Literal::String(s) => write!(f, "{s}"),
        }
    }
}

impl fmt::Display for TonsOfTokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = Vec::new();
        let indent = INDENT.load(Ordering::Relaxed);
        if DO_INDENT.load(Ordering::Relaxed) {
            out.push((0..indent).into_iter().map(|_| '\t').collect::<String>());
        }

        match &self {
            &Token::Ident(ident) => out.push(format!("{ident}")),

            &Token::Assignment {
                mutable,
                type_annotation,
                ident,
                value,
            } => {
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!(
                    "Assignment{mutable_fmt}: {ident} {type_annotation_fmt} = {value}",
                    mutable_fmt = if *mutable { " (mut)" } else { "" },
                    type_annotation_fmt = match type_annotation {
                        None => "[unknown]".to_string(),
                        Some(token) => format!("[{token}]"),
                    }
                ))
            }

            &Token::Param {
                mutable,
                type_annotation,
                ident,
            } => {
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!(
                    "{type_annotation} {ident} {mutable_fmt}",
                    mutable_fmt = if *mutable { " (mut)" } else { "" },
                ))
            }

            &Token::Literal(literal) => out.push(format!("{literal}")),

            &Token::Op(op) => out.push(format!("{op}")),

            &Token::IfStmt { cond } => out.push(format!("Cond: {cond}")),

            &Token::Pair(p1, p2) => {
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!("<{p1}"));
                DO_INDENT.store(true, Ordering::Relaxed);
                if format!("{p2}").trim() == "" {
                    out.push(format!(">"));
                } else {
                    out.push(format!(";\n{p2}>"));
                }
            }

            &Token::CondPair(p1, p2, p3) => {
                out.push((0..indent).into_iter().map(|_| '\t').collect::<String>());
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!("<<?{p1};\n"));
                DO_INDENT.store(true, Ordering::Relaxed);
                INDENT.store(INDENT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                out.push(format!("{p2}\n"));
                if format!("{p3}").trim() == "" {
                    out.push((0..indent).into_iter().map(|_| '\t').collect::<String>());
                    out.push(format!("?>"));
                    DO_INDENT.store(false, Ordering::Relaxed);
                } else {
                    out.push(format!("?>\n"));
                }

                INDENT.store(INDENT.load(Ordering::Relaxed) - 1, Ordering::Relaxed);
                out.push(format!("{p3}>"));
            }

            &Token::FnPair(p1, p2, p3) => {
                out.push((0..indent).into_iter().map(|_| '\t').collect::<String>());
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!("<<λ{p1};\n"));
                DO_INDENT.store(true, Ordering::Relaxed);
                INDENT.store(INDENT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                out.push(format!("{p2}\n"));
                if format!("{p3}").trim() == "" {
                    out.push((0..indent).into_iter().map(|_| '\t').collect::<String>());
                    out.push(format!("λ>"));
                    DO_INDENT.store(false, Ordering::Relaxed);
                } else {
                    out.push(format!("λ>\n"));
                }

                INDENT.store(INDENT.load(Ordering::Relaxed) - 1, Ordering::Relaxed);
                out.push(format!("{p3}>"));
            }

            &Token::Method(operator, method) => {
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!("<{operator} THEN \n"));
                DO_INDENT.store(true, Ordering::Relaxed);
                out.push(format!("{method}>"));
            }

            &Token::None => (),
        }

        write!(f, "{}", out.join(""))
    }
}
