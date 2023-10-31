use cmdstan::argument_tree::{ArgumentTreeBuilder, Data, OutputBuilder, Random};
use cmdstan::control::*;
use cmdstan::sample::{HmcBuilder, Metric, NutsBuilder, SampleBuilder};
use cmdstan::*;
use std::path::PathBuf;

fn main() {
    // Typically, one would not use the current working directory;
    // this example utilizes the current working directory so that it
    // may be run from within the repository using `cargo run
    // --example bernoulli` (from crate root)
    let mut path = PathBuf::from(std::env::current_dir().unwrap());
    path.push("examples");
    path.push("bernoulli");

    let mut model_file = path.clone();
    model_file.push("bernoulli.stan");

    let mut data_file = path.clone();
    data_file.push("bernoulli.data.json");
    let mut output_file = path.clone();
    output_file.push("output.csv");

    let stan_program = StanProgram::from(model_file);

    let workspace = Workspace {
        model_name: "bernoulli".to_string(),
        directory: path.to_string_lossy().to_string(),
        stan_program,
    };
    println!("{:#?}", workspace.setup());

    // These options are mostly nonsense, but the point is that
    // one can conveniently specify them.
    let tree = ArgumentTreeBuilder::new()
        .method(
            SampleBuilder::new()
                .num_samples(1234)
                .num_warmup(789)
                .save_warmup(false)
                .thin(2)
                .algorithm(
                    HmcBuilder::new()
                        .engine(NutsBuilder::new().max_depth(100).build())
                        .metric(Metric::DenseE)
                        .build(),
                )
                .num_chains(4)
                .build(),
        )
        .data(Data {
            file: data_file.to_string_lossy().to_string(),
        })
        .id(2)
        .init("1".to_string())
        .random(Random { seed: 12345 })
        .output(
            OutputBuilder::new()
                .sig_figs(4)
                .file(output_file.to_string_lossy().to_string())
                .build(),
        )
        .num_threads(48)
        .build();

    // Of course, one need not rely on an environment variable, but
    // this makes the example as portable as can be.
    let cmdstan_home =
        std::env::var("CMDSTAN_HOME").expect("CMDSTAN_HOME environment variable not set!");
    let control = Control::new(&cmdstan_home, &workspace.model());
    // If a binary already exists, calling compile is somewhat strange, thus,
    // we check if there is a working rather than re-compiling by default.
    if !control.executable_works().unwrap_or(false) {
        println!("{:#?}", control.compile());
    }
    // This will print both stdout and stderr from the method call;
    // typically, one may wish to log this. This crate may provide a
    // logging flag in the future.
    println!("{:#?}", control.call_executable(&tree));

    println!("{:#?}", control.diagnose(&tree));
    println!("{:#?}", control.stansummary(&tree, None));

    let mut csv_filename = path.clone();
    csv_filename.push("stansummary.csv");
    let summary_opts = StanSummaryOptions {
        autocorr: None,
        csv_filename: Some(csv_filename.to_string_lossy().to_string()),
        percentiles: Some(vec![5, 25, 50, 75, 95]),
        sig_figs: Some(6),
    };
    println!("{:#?}", control.stansummary(&tree, Some(summary_opts)));
}
