use cmdstan::argument_tree::{ArgumentTree, Data, Output};
use cmdstan::CmdStanModel;
use cmdstan::{
    optimize::OptimizeBuilder, pathfinder::PathfinderBuilder, sample::SampleBuilder,
    variational::VariationalBuilder,
};
use std::{env, path::PathBuf};

fn main() {
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

    let tree = ArgumentTree::builder()
        .data(Data {
            file: data_file.into(),
        })
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

    // Of course, one need not rely on an environment variable, but
    // this makes the example as portable as can be.
    let cmdstan = env::var("CMDSTAN_HOME").expect("CMDSTAN_HOME environment variable not set!");
    let model = CmdStanModel::new(&cmdstan, &model_file);
    // If a binary already exists, calling compile is somewhat strange, thus,
    // we check if there is a working rather than re-compiling by default.
    if !model.executable_works().unwrap_or(false) {
        println!("{:#?}", model.compile());
    }

    for tree in trees {
        match model.call_executable(&tree) {
            Ok(output) => {
                println!("{:#?}", output.output());
            }
            Err(e) => {
                println!("Something seems to have gone wrong...\n{:#?}", e);
            }
        }
    }
}
