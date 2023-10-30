use cmdstan::argument_tree::{ArgumentTreeBuilder, Data, OutputBuilder, Random};
use cmdstan::sample::{HmcBuilder, Metric, NutsBuilder, SampleBuilder};
use cmdstan::*;
use std::path::PathBuf;

fn main() {
    let mut path = PathBuf::from(std::env::current_dir().unwrap());
    path.push("examples");
    path.push("bernoulli");

    let mut model_file = path.clone();
    model_file.push("bernoulli.stan");

    let mut data_file = path.clone();
    data_file.push("bernoulli.data.json");

    let stan_program = StanProgram::from(model_file);

    let workspace = Workspace {
        model_name: "bernoulli".to_string(),
        directory: path.to_string_lossy().to_string(),
        stan_program,
    };
    println!("{:#?}", workspace.setup());

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
        .output(OutputBuilder::new().sig_figs(4).build())
        .num_threads(48)
        .build();

    let cmdstan_home =
        std::env::var("CMDSTAN_HOME").expect("CMDSTAN_HOME environment variable not set!");
    let control = Control::new(&cmdstan_home, &workspace.model());
    if !control.executable_works().unwrap_or(false) {
        println!("{:#?}", control.compile());
    }
    println!("{:#?}", control.call_executable(&tree));

    println!("{:#?}", control.diagnose(&tree));
    println!("{:#?}", control.stansummary(&tree, None));

    let summary_opts = StanSummaryOptions {
        autocorr: None,
        csv_filename: Some("stansummary.csv".to_string()),
        percentiles: Some(vec![5, 25, 50, 75, 95]),
        sig_figs: Some(6),
    };
    println!("{:#?}", control.stansummary(&tree, Some(summary_opts)));
}
