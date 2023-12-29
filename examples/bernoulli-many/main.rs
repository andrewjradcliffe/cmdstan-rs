use cmdstan::argtree::{ArgTree, Data, Output};
use cmdstan::*;
use cmdstan::{
    optimize::OptimizeBuilder, pathfinder::PathfinderBuilder, sample::SampleBuilder,
    variational::VariationalBuilder,
};
use std::{env, path::PathBuf};

fn main() {
    // Of course, one need not rely on an environment variable, but
    // this makes the example as portable as can be.
    let path = env::var("CMDSTAN").expect("CMDSTAN environment variable not set!");
    let cmdstan = CmdStan::try_from(path.as_ref()).expect("Something went wrong with CmdStan");

    // Typically, one would not use the current working directory;
    // this example utilizes the current working directory so that it
    // may be run from within the repository using `cargo run
    // --example bernoulli` (from crate root)
    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("examples");
    path.push("bernoulli");

    let model_file = path.join("bernoulli.stan");
    let data_file = path.join("bernoulli.data.json");
    path.set_file_name("bernoulli-many");

    let program = StanProgram::try_from(model_file.as_ref()).expect("Stan program does not exist");
    let model = cmdstan
        .compile::<[_; 0], &str>(&program, [])
        .expect("Something went wrong with compilation");

    let tree = ArgTree::builder()
        .data(Data::builder().file(data_file))
        .build();

    // This supplies the defaults for each method
    let methods = [
        SampleBuilder::new().build(),
        OptimizeBuilder::new().build(),
        VariationalBuilder::new().build(),
        PathfinderBuilder::new().build(),
    ];
    let out_files = [
        "sample.csv",
        "optimize.csv",
        "variational.csv",
        "pathfinder.csv",
    ];
    let trees = methods
        .into_iter()
        .zip(out_files.into_iter())
        .map(move |(method, file)| {
            let mut tree = tree.clone();
            tree.method = method;
            tree.output = Output::builder().file(path.join(file)).build();
            tree
        });

    for tree in trees {
        match model.call(&tree) {
            Ok(output) => {
                println!("{:#?}", output.output());
            }
            Err(e) => {
                println!("Something seems to have gone wrong...\n{:#?}", e);
            }
        }
    }
}
