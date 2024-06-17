use assert_cmd::prelude::*;
use assert_fs::fixture::FileWriteStr;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn array_index() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("main.ed")?;
    file.write_str("let x := [1, 2, 3, 4, 32 + 12];\nlet y := x[2];\nyield y;")?;

    let mut cmd = Command::cargo_bin("lead")?;
    cmd.arg("run").arg(file.path());
    cmd.assert().success().stdout(predicate::str::contains("3"));
    Ok(())
}
