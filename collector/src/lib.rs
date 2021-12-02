use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn collect(lib: &str, output: Option<PathBuf>) -> Result<()> {
    //TODO: Do something with files?
    if !PathBuf::from(lib).is_dir() {
        return Err(anyhow!("The library to collect must be a directory"));
    }

    /*let entries = lib
    .read_dir()?
    .filter(|dir| dir.is_ok())
    .map(|dir| dir.unwrap())
    .collect::<Vec<DirEntry>>();*/

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::collect;

    #[test]
    fn collect_test_1() {
        collect(PathBuf::from("../core/lib").to_str().unwrap(), None).unwrap();
    }
}
