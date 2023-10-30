use cmdstan::argument_tree::{ArgumentTreeBuilder, Data, OutputBuilder, Random};
use cmdstan::control::*;
use cmdstan::sample::{HmcBuilder, Metric, NutsBuilder, SampleBuilder};
use cmdstan::*;

fn main() {
    // let model = "/nfs/site/home/aradclif/aradclif/org/org-linux/stan/examples/bernoulli/bernoulli";

    // let bigcmd = "method=sample num_samples=1000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=10 metric=diag_e stepsize=1 stepsize_jitter=0 num_chains=4 data file=bernoulli.data.json id=2 init=1 random seed=12345 output file=output.csv refresh=100 sig_figs=-1 num_threads=1";

    // let stan_program = StanProgram::from(
    //     "data {
    //        int<lower=0> N;
    //        array[N] int<lower=0,upper=1> y;
    //      }
    //      parameters {
    //        real<lower=0,upper=1> theta;
    //      }
    //      model {
    //        theta ~ beta(1,1);   //uniform prior on interval 0,1
    //        y ~ bernoulli(theta);
    //      }
    //      ",
    // );
    let stan_program = StanProgram::from(std::path::Path::new(
        "/nfs/site/home/aradclif/aradclif/org/org-linux/stan/examples/bernoulli/bernoulli.stan",
    ));

    let workspace = Workspace {
        model_name: "bernoulli".to_string(),
        directory: "/nfs/site/home/aradclif/aradclif/org/org-linux/stan/examples/bernoulli/inner4"
            .to_string(),
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
            file: "/nfs/site/home/aradclif/aradclif/org/org-linux/stan/examples/bernoulli/bernoulli.data.json".to_string(),
        })
        .id(2)
        .init("1".to_string())
        .random(Random { seed: 12345 })
        .output(OutputBuilder::new().sig_figs(4).build())
        .num_threads(48)
        .build();
    let bigcmd = tree.command_string();
    println!("{}", bigcmd);

    // let con = Control::new("/nfs/site/home/aradclif/aradclif/cmdstan-2.33.1", model);
    let con = Control::new(
        "/nfs/site/home/aradclif/aradclif/cmdstan-2.33.1",
        &workspace.model(),
    );
    // let con = Control::try_from(model).unwrap();
    if !con.executable_works().unwrap_or(false) {
        println!("{:#?}", con.compile());
    }
    println!("{:#?}", con.call_executable(&tree));

    println!("{:#?}", con.diagnose(&tree));
    println!("{:#?}", con.stansummary(&tree, None));

    // let summary_opts = StanSummaryOptions::new()
    //     .csv_filename("hello2.csv".to_string())
    //     .sig_figs(6)
    //     .percentiles(vec![5, 25, 50, 75, 95]);
    let summary_opts = StanSummaryOptions {
        autocorr: None,
        csv_filename: Some("hello3.csv".to_string()),
        percentiles: Some(vec![5, 25, 50, 75, 95]),
        sig_figs: Some(6),
    };
    println!("{:#?}", con.stansummary(&tree, Some(summary_opts)));

    // env::set_var(
    //     "CMDSTAN_HOME",
    //     "/nfs/site/home/aradclif/aradclif/cmdstan-2.33.1",
    // );
    // if let Ok(home) = env::var("CMDSTAN_HOME") {
    //     println!("CMDSTAN_HOME={}", home);
    //     if let Ok(()) = env::set_current_dir(home) {
    //         let output = Command::new("make")
    //             .arg(model)
    //             .output()
    //             .expect("Failed to execute command");
    //         println!("{:#?}", output);
    //         let stdout = String::from_utf8(output.stdout).unwrap();
    //         if stdout.contains("is up to date.\n") {
    //             if let Ok(()) = env::set_current_dir(model.rsplit_once('/').unwrap().0) {
    //                 // Different ways to create the same command
    //                 // let output = Command::new(model)
    //                 //     .arg("method=sample")
    //                 //     .arg("help")
    //                 //     .output()
    //                 //     .expect("Failed to execute command");
    //                 // println!("{:#?}", output);

    //                 // let output = Command::new(model)
    //                 //     .args(["method=sample", "help"])
    //                 //     .output()
    //                 //     .expect("Failed to execute command");
    //                 // println!("{:#?}", output);
    //                 // let output = Command::new(model)
    //                 //     .args("method=sample help".split_whitespace())
    //                 //     .output()
    //                 //     .expect("Failed to execute command");
    //                 // println!("{:#?}", output);

    //                 // Test on a larger argument. Works.
    //                 let output = Command::new(model)
    //                     .args(bigcmd.split_whitespace())
    //                     .output()
    //                     .expect("Failed to execute big command");
    //                 println!("{:#?}", output);
    //             }
    //         }
    //     }
    // }
}
