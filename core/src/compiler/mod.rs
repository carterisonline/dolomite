use std::convert::TryInto;
use std::fs;

use anyhow::Result;
use log::info;

use crate::compiler::translate::translate_file;
use crate::parser::Token;

pub mod translate;

macro_rules! command {
    ($cmd: expr) => {
        if cfg!(target_os = "windows") {
            std::process::Command::new("cmd")
                .arg("/C")
                .arg($cmd)
                .output()
        } else {
            std::process::Command::new("sh")
                .arg("-c")
                .arg($cmd)
                .output()
        }
    };
}

#[derive(Default)]
pub struct Program {
    name: String,
}

impl Program {
    pub fn new(name: &str) -> Result<Self> {
        Ok(Self {
            name: name.try_into()?,
            ..Default::default()
        })
    }

    pub fn compile(&mut self, code: Token) -> Result<()> {
        let translated = translate_file(box code)?;
        info!("Compiled to {translated}");
        fs::write(&format!("./{}.v", self.name), translated)?;
        command!(format!("v ./{}.v", self.name))?;
        // command!(format!("rm ./{}.v", self.name))?;
        Ok(())
    }
}
