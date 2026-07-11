use std::process::Command;

fn main() {
    let commit = match Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(_) => String::from("unknown"),
    };

    println!("cargo::rustc-env=GIT_HASH={commit}");
}
