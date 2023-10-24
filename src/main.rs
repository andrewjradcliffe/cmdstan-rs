use cmdstan::*;
use std::env;
use std::process::Command;

static STANFILE: &str =
    "/nfs/orto/proj/tapeout/cit_dev16/aradclif/cmdstan/examples/bernoulli/bernoulli.stan";

fn main() {
    let output = Command::new("echo")
        .arg("Hello world")
        .output()
        .expect("Failed to execute command");

    assert_eq!(b"Hello world\n", output.stdout.as_slice());

    let cwd = env::current_dir();
    println!("{:#?}", cwd);

    let output = Command::new("pwd")
        .output()
        .expect("Failed to execute command");
    println!("{:#?}", output);

    // for (var, val) in env::vars() {
    //     println!("{var} = {val}");
    // }

    if let Ok(home) = env::var("CMDSTAN_HOME") {
        let maybe = env::set_current_dir(home);
        println!("{:#?}", maybe);
        let cwd = env::current_dir();
        println!("cwd is now: {:#?}", cwd);
    }

    let output = Command::new("which")
        .arg("make")
        .output()
        .expect("Failed to execute command");
    println!("{:#?}", output);

    if let Ok(cmdstan) = env::var("CMDSTAN_HOME") {
        env::set_current_dir(cmdstan).expect("Unable to change directories");
        let output = Command::new("bin/stansummary")
            .arg("-h")
            .output()
            .expect("Failed to execute command");
        println!("{:#?}", output);
    } else {
        println!("Unable to find cmdstan");
    }

    let output = Command::new("mkdir")
        .arg("testing-mkdir")
        .output()
        .expect("Failed to execute command");
    println!("{:#?}", output);

    let argument_tree = ArgumentTree::default();
    let method = Method::default();
    println!("{:#?}", method);

    println!("{:#?}", argument_tree);
}
