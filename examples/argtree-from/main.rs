use cmdstan::argtree::ArgTree;

fn main() {
    let limited_cli = "sample data file=bernoulli.data.json random seed=589886520";
    let t1 = limited_cli.parse::<ArgTree>().unwrap();
    println!("{:#?}", t1);

    let full_cli = "method=sample num_samples=1000 num_warmup=1000 save_warmup=0 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=10 metric=diag_e stepsize=1 stepsize_jitter=0 num_chains=1 id=1 data file=bernoulli.data.json init=2 random seed=589886520 output file=output.csv refresh=100 sig_figs=-1 profile_file=profile.csv num_threads=1";
    let t2 = full_cli.parse::<ArgTree>().unwrap();

    assert_eq!(t1, t2);
    let t3 = ArgTree::from_reader(SAMPLE_CSV.as_bytes())
        .unwrap()
        .unwrap();

    assert_eq!(t1, t3);

    let t4 = ArgTree::from_reader(STDOUT.as_bytes())
        .unwrap()
        .unwrap();
    assert_eq!(t1, t4);
}

static SAMPLE_CSV: &str = "# stan_version_major = 2
# stan_version_minor = 33
# stan_version_patch = 0
# model = neural_model
# start_datetime = 2023-11-29 16:52:51 UTC
# method = sample (Default)
#   sample
#     num_samples = 1000 (Default)
#     num_warmup = 1000 (Default)
#     save_warmup = 0 (Default)
#     thin = 1 (Default)
#     adapt
#       engaged = 1 (Default)
#       gamma = 0.050000000000000003 (Default)
#       delta = 0.80000000000000004 (Default)
#       kappa = 0.75 (Default)
#       t0 = 10 (Default)
#       init_buffer = 75 (Default)
#       term_buffer = 50 (Default)
#       window = 25 (Default)
#     algorithm = hmc (Default)
#       hmc
#         engine = nuts (Default)
#           nuts
#             max_depth = 10 (Default)
#         metric = diag_e (Default)
#         metric_file =  (Default)
#         stepsize = 1 (Default)
#         stepsize_jitter = 0 (Default)
#     num_chains = 1 (Default)
# id = 1 (Default)
# data
#   file = bernoulli.data.json
# init = 2 (Default)
# random
#   seed = 589886520 (Default)
# output
#   file = output.csv (Default)
#   diagnostic_file =  (Default)
#   refresh = 100 (Default)
#   sig_figs = -1 (Default)
#   profile_file = profile.csv (Default)
# num_threads = 1 (Default)
# stanc_version = stanc3 v2.33.1
# stancflags =
lp__,accept_stat__,stepsize__,treedepth__,n_leapfrog__,divergent__,energy__,theta
# Adaptation terminated
# Step size = 0.911297
# Diagonal elements of inverse mass matrix:
# 0.566254
-8.07514,0.722434,0.911297,2,3,0,8.62107,0.0898933
-7.23378,0.964012,0.911297,1,3,0,7.95058,0.142034
-7.18599,0.90941,0.911297,1,3,0,8.12148,0.37709
-6.85678,1,0.911297,2,3,0,7.10835,0.19493
-7.18578,0.905922,0.911297,1,3,0,7.51652,0.377057";

static STDOUT: &str = "method = sample (Default)
  sample
    num_samples = 1000 (Default)
    num_warmup = 1000 (Default)
    save_warmup = 0 (Default)
    thin = 1 (Default)
    adapt
      engaged = 1 (Default)
      gamma = 0.050000000000000003 (Default)
      delta = 0.80000000000000004 (Default)
      kappa = 0.75 (Default)
      t0 = 10 (Default)
      init_buffer = 75 (Default)
      term_buffer = 50 (Default)
      window = 25 (Default)
    algorithm = hmc (Default)
      hmc
        engine = nuts (Default)
          nuts
            max_depth = 10 (Default)
        metric = diag_e (Default)
        metric_file =  (Default)
        stepsize = 1 (Default)
        stepsize_jitter = 0 (Default)
    num_chains = 1 (Default)
id = 1 (Default)
data
  file = bernoulli.data.json
init = 2 (Default)
random
  seed = 589886520 (Default)
output
  file = output.csv (Default)
  diagnostic_file =  (Default)
  refresh = 100 (Default)
  sig_figs = -1 (Default)
  profile_file = profile.csv (Default)
num_threads = 1 (Default)";
