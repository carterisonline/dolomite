//TODO: Follow DRY principles (the code sucks)

#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(in_band_lifetimes)]
#![feature(format_args_capture)]

pub mod compiler;
pub mod parser;

#[macro_export]
macro_rules! attempt {
    ($part: literal from $src: expr) => {
        if log::log_enabled!(log::Level::Info) {
            if (&$src).trim().len() > 0 {
                log::info!("ATTEMPT {} from {}", $part, &$src);
            }
        }
    };
}

#[macro_export]
macro_rules! got {
    ($part: literal from $src: expr) => {
        if log::log_enabled!(log::Level::Info) {
            if (&$src).trim().len() > 0 {
                log::info!("GOT {} from {}", $part, &$src);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::compiler::Program;
    use crate::parser::parse;

    #[test]
    fn parse_test_1() {
        dolomite_logger::init();
        let src = std::fs::read_to_string("dl/parse-test-1.dl").unwrap();
        let parsed = parse(&src);

        let mut compiler = Program::new("parse_test_1").unwrap();
        compiler.compile(parsed).unwrap();
    }

    /*
    #[test]
    fn parse_test_2() {
        env_logger::init();
        let src = std::fs::read_to_string("dl/parse-test-2.dl").unwrap();
        let parsed = parse(&src);
        println!("(tests::parse_test_2)\nParsed Tokens:\n");
        println!("{:?}", parsed);
        match parsed {
            Token::Pair(p1, _) => {
                println!("{p1}");
                println!("{}", translate_file(p1).unwrap());
            }
            _ => unimplemented!(),
        }
    }*/

    /*#[test]
    fn langtons_ant() {
        let src = std::fs::read_to_string("dl/langtons-ant.dl").unwrap();
        let parsed = parse(&src);
        println!("(tests::langtons_ant)\nParsed Tokens:\n");
        println!("{parsed}");
    }*/
}
