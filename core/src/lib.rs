#![feature(box_syntax)]
#![feature(in_band_lifetimes)]
#![feature(format_args_capture)]

pub mod parser;

#[cfg(test)]
mod tests {
    use crate::parser::literals::{Literal, StrictNumber, VagueLiteral};
    use crate::parser::ops::Op;
    use crate::parser::{parse, Token};

    #[test]
    fn parse_test_1() {
        let src = std::fs::read_to_string("dl/parse-test-1.dl").unwrap();
        let parsed = parse(&src);
        println!("(tests::parse_test_1)\nParsed Tokens:\n");
        println!("{parsed}");
        assert_eq!(
            parsed,
            Token::Pair(
                box Token::Assignment {
                    mutable: true,
                    type_annotation: None,
                    ident: box Token::Ident("output".into()),
                    value: box Token::Literal(Literal::Vague(VagueLiteral::Integer("0".into())))
                },
                box Token::Pair(
                    box Token::Assignment {
                        mutable: false,
                        type_annotation: Some(box Token::Ident("byte".into())),
                        ident: box Token::Ident("dice".into()),
                        value: box Token::Method(
                            box Token::Op(Op::Subtract(
                                box Token::Literal(Literal::Number(StrictNumber::Byte(1))),
                                box Token::Literal(Literal::Number(StrictNumber::Byte(1)))
                            )),
                            box Token::Method(
                                box Token::Ident("randomize".into()),
                                box Token::Ident("normalize".into())
                            )
                        )
                    },
                    box Token::CondPair(
                        box Token::IfStmt {
                            cond: box Token::Op(Op::Gte(
                                box Token::Ident("dice".into()),
                                box Token::Literal(Literal::Vague(VagueLiteral::Integer(
                                    "127".into()
                                )))
                            ))
                        },
                        box Token::Pair(
                            box Token::Assignment {
                                mutable: false,
                                type_annotation: None,
                                ident: box Token::Ident("output".into()),
                                value: box Token::Literal(Literal::Vague(VagueLiteral::Integer(
                                    "1".into()
                                )))
                            },
                            box Token::None
                        ),
                        box Token::Method(
                            box Token::Ident("output".into()),
                            box Token::Ident("print".into())
                        )
                    )
                )
            )
        );
    }
}
