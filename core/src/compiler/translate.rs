use anyhow::{anyhow, Result};
use log::info;

use crate::parser::literals::{Literal, StrictNumber, VagueLiteral};
use crate::parser::ops::Op;
use crate::parser::Token;

const CORE_PRELUDE: &str = r#"

"#;

macro_rules! you_asked {
    ($req: expr, $didntget: expr) => {
        format!(
            "You asked to parse {} but you didn't give me {}?!?! >:(",
            $req, $didntget
        )
    };

    ($req: expr) => {
        you_asked!($req, $req)
    };
}

macro_rules! ops {
    ($matcher: ident, [$($op: ident = $former: ident $op_str: literal $latter: ident),+]) => {
        match $matcher {
            $(Op::$op($former, $latter) => Ok(format!(
                "({} {} {})",
                translate($former)?,
                $op_str,
                translate($latter)?
            )),)+
        }

    }
}

fn transform_literal(lit: &str) -> Result<String> {
    return Ok(match lit {
        "byte" => "u8",
        "small" => "u16",
        "medium" => "u32",
        "large" => "u64",
        "bytesigned" => "i8",
        "smallsigned" => "i16",
        "mediumsigned" => "i32",
        "largesigned" => "i64",
        "mediumfloat" => "f32",
        "largefloat" => "f64",
        _ => return Err(anyhow!("Failed to parse literal identifier")),
    }
    .to_string());
}

pub fn translate_file(source: Box<Token>) -> Result<String> {
    Ok(format!("{CORE_PRELUDE}\n{}", translate(source)?))
}

fn translate(source: Box<Token>) -> Result<String> {
    info!("translate {source}");
    match source.clone() {
        box Token::Assignment {
            mutable,
            type_annotation,
            ident,
            value,
        } => match value {
            box Token::FnPair(_, _, _) => gen_function(source),
            val => Ok(format!(
                "{}{} := {}{}{}",
                if mutable {
                    info!("\tmut assignment from {source}");
                    "mut "
                } else {
                    info!("{:?}", val);
                    info!("\tassignment from {source}");
                    ""
                },
                translate(ident)?,
                if let Some(t) = type_annotation.clone() {
                    format!("{}(", translate(t)?)
                } else {
                    String::new()
                },
                translate(val)?,
                if type_annotation.is_some() { ")" } else { "" },
            )),
        },

        box Token::MethodUnit(name, args) => match (name, args) {
            (box Token::Ident(name_raw), box Token::Array(args_raw)) => {
                info!("\tmultiarg methodunit from {source}");
                Ok(format!(
                    "{name_raw}({})",
                    args_raw
                        .iter()
                        .map(|s| format!("{s}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                ))
            }

            (box Token::Ident(name_raw), arg) => {
                info!("\tsinglearg methodunit from {source}");
                Ok(format!(
                    "{}({})",
                    translate(box Token::Ident(name_raw))?,
                    translate(arg)?
                ))
            }

            _ => unimplemented!(),
        },

        box Token::Literal(lit) => match lit {
            Literal::Vague(vague) => match vague {
                // LITERALLY NEVER DO THIS EVER
                VagueLiteral::Integer(i) => {
                    info!("\tvagueint from {source}");
                    Ok(format!("{i}"))
                }
                _ => unimplemented!(),
            },
            Literal::String(s) => {
                info!("\tstring form {source}");
                Ok(format!("\"{s}\""))
            }
            Literal::Number(strictnum) => match strictnum {
                StrictNumber::Byte(i) => {
                    info!("\tbyte from {source}");
                    Ok(format!("u8({i})"))
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        },

        box Token::Pair(former, latter) => {
            info!("\tpair from {source}");
            Ok(format!("{}\n{}", translate(former)?, translate(latter)?))
        }

        box Token::CondPair(cond, block, latter) => {
            info!("\tcondpair from {source}");
            Ok(format!(
                "if {} {{\n{}\n}}\n {}",
                translate(cond)?,
                translate(block)?,
                translate(latter)?
            ))
        }

        box Token::IfStmt { cond } => Ok(format!("{}", translate(cond)?)),

        box Token::Op(op) => Ok(format!("{}", translate_op(op)?)),

        box Token::Ident(ident) => {
            info!("\tident from {source}");
            if let Ok(transformed) = transform_literal(&ident) {
                return Ok(transformed);
            }
            Ok(format!("{ident}"))
        }

        box Token::None => {
            info!("\tnone from {source}");
            Ok(String::new())
        }

        token => {
            info!("\t{:?} is unimplemented!", token);
            unimplemented!();
        }
    }
}

// rust macros :chef's kiss:
fn translate_op(op: Op) -> Result<String> {
    ops!(op, [
        Add = former "+" latter,
        Subtract = former "-" latter,
        Eq = former "==" latter,
        Neq = former "!=" latter,
        Gt = former ">" latter,
        Lt = former "<" latter,
        Gte = former ">=" latter,
        Lte = former "<=" latter
    ])
}

fn gen_function(assignment: Box<Token>) -> Result<String> {
    info!("generating a function from {assignment}");
    match assignment {
        box Token::Assignment {
            mutable: _,
            type_annotation,
            ident,
            value,
        } => match value {
            box Token::FnPair(args, body, after) => {
                return Ok(format!(
                    //TODO: Change for pub visibility levels
                    r#"fn {name}({fnargs}) {ret} {{
                        {inner}
                    }}

                    {latter}
                "#,
                    inner = translate(body)?,
                    name = match ident {
                        box Token::Ident(s) => s,
                        _ =>
                            return Err(anyhow!(
                                "I need a name for the function, or it wasn't valid."
                            )),
                    },
                    fnargs = if args.0.is_empty() {
                        String::new()
                    } else {
                        args.0
                            .iter()
                            .map(|id| match id {
                                Token::Param {
                                    type_annotation,
                                    ident,
                                    mutable,
                                } => match (type_annotation, ident) {
                                    (box Token::Ident(ty), box Token::Ident(param)) => {
                                        format!(
                                            "{}{param} {}",
                                            if *mutable { "mut " } else { "" },
                                            transform_literal(ty).unwrap()
                                        )
                                    }
                                    _ => unimplemented!(),
                                },
                                _ => unimplemented!(),
                            })
                            .collect::<Vec<String>>()
                            .join(", ")
                    },
                    ret = match type_annotation {
                        Some(box Token::Ident(id)) => {
                            transform_literal(&id)?
                        }

                        None => String::new(),

                        _ => return Err(anyhow!("failed to parse return type for function")),
                    },
                    latter = translate(after)?
                )
                .lines()
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
                .join("\n"));
            }
            _ => return Err(anyhow!(you_asked!("a function"))),
        },
        _ => {
            return Err(anyhow!(you_asked!(
                "a function's assignment",
                "an assignment"
            )))
        }
    }
}
