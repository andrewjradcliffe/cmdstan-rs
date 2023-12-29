use cmdstan::argtree::{ArgTree, Data, Output, Random};
use cmdstan::sample::{HmcBuilder, Metric, NutsBuilder, SampleBuilder};
use cmdstan::stansummary::StanSummaryOptions;
use cmdstan::*;
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
    let output_file = path.join("output.csv");

    let program = StanProgram::try_from(model_file.as_ref()).expect("Stan program does not exist");
    let model = cmdstan
        .compile::<[_; 0], &str>(&program, [])
        .expect("Something went wrong with compilation");

    // These options are intentionally verbose; the point is that
    // one can conveniently specify them.
    let tree = ArgTree::builder()
        .method(
            SampleBuilder::new()
                .num_samples(1234)
                .num_warmup(789)
                .save_warmup(false)
                .thin(2)
                .algorithm(
                    HmcBuilder::new()
                        .engine(NutsBuilder::new().max_depth(100))
                        .metric(Metric::DenseE),
                )
                .num_chains(4),
        )
        .data(Data::builder().file(data_file))
        .id(2)
        .init("1")
        .random(Random::builder().seed(12345))
        .output(
            Output::builder()
                .sig_figs(4)
                .file(output_file)
                .profile_file(path.join("profile.csv")),
        )
        .num_threads(48)
        .build();

    // Automatically logs stdout/stderr
    let output = model.call(&tree).expect("Some problem with the executable");

    let opts = StanSummaryOptions::builder()
        .csv_filename(path.join("summary.csv"))
        .percentiles([2.5, 50.0, 97.5])
        .build();

    let summary = cmdstan
        .stansummary(&output, opts)
        .expect("stansummary problem");
    println!("{}", String::from_utf8_lossy(&summary.stdout));
    println!("{:#?}", summary);
}
