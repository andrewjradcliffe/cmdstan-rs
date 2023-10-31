use cmdstan::argument_tree::{ArgumentTreeBuilder, Data, OutputBuilder, Random};
use cmdstan::sample::{HmcBuilder, Metric, NutsBuilder, SampleBuilder};
use cmdstan::*;
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
    let output_file = path.join("output.csv");

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
        .init("1")
        .random(Random { seed: 12345 })
        .output(
            OutputBuilder::new()
                .sig_figs(4)
                .file(output_file.to_string_lossy())
                .build(),
        )
        .num_threads(48)
        .build();

    // Of course, one need not rely on an environment variable, but
    // this makes the example as portable as can be.
    let cmdstan = env::var("CMDSTAN_HOME").expect("CMDSTAN_HOME environment variable not set!");
    let model = CmdStanModel::new(&cmdstan, &model_file);
    // If a binary already exists, calling compile is somewhat strange, thus,
    // we check if there is a working rather than re-compiling by default.
    if !model.executable_works().unwrap_or(false) {
        println!("{:#?}", model.compile());
    }
    // This will print both stdout and stderr from the method call;
    // typically, one may wish to log this. This crate may provide a
    // logging flag in the future.
    match model.call_executable(&tree) {
        Ok(output) => {
            println!("{:#?}", output.output());
            println!("{:#?}", output.diagnose());
            println!("{:#?}", output.stansummary(None));
            println!("{:#?}", output.write_output(Some(path.join("log.txt"))));

            let csv_filename = path.join("stansummary.csv");
            let summary_opts = StanSummaryOptions {
                autocorr: None,
                csv_filename: Some(csv_filename.to_string_lossy().to_string()),
                percentiles: Some(vec![5, 25, 50, 75, 95]),
                sig_figs: Some(6),
            };
            println!("{:#?}", output.stansummary(Some(summary_opts)));
        }
        Err(e) => {
            println!("Something seems to have gone wrong...\n{:#?}", e);
        }
    }
}
