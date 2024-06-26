use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn first_ten_fibs() -> [u32; 10] {
    let mut fibs = [0; 10];
    fibs.iter_mut().fold((1, 1), |acc: (u32, u32), ele| {
        *ele = acc.0;
        (acc.1, *ele + acc.1)
    });
    fibs
}

#[test]
fn fib() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("leadc")?;
    cmd.arg("run")
        .arg("../examples/fib.ed")
        .arg("-l")
        .arg("../logs/examples-test.log");

    cmd.assert().success().stdout(predicate::str::diff(format!(
        "{}\n",
        first_ten_fibs().map(|ele| format!("{ele}")).join("\n")
    )));
    Ok(())
}
