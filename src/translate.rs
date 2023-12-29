use std::ffi::OsString;

pub use translate_derive::*;

/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait Translate: private::Sealed {
    /// Write `self` to `s` as a statement in command line language.
    /// If `s` has sufficient capacity to hold the result, this will
    /// not allocate.
    fn write_stmt(&self, s: &mut OsString);
    /// Write `self` to `s` as a tree, with offset (from left) of `n`.
    /// If `s` has sufficient capacity to hold the result, this will
    /// not allocate.
    fn write_tree_offset(&self, n: usize, s: &mut OsString);
    /// Translate `self` to command line arguments and append to `v`.
    fn append_args(&self, v: &mut Vec<OsString>);

    /// Write `self` to `s` as a tree.
    /// If `s` has sufficient capacity to hold the result, this will
    /// not allocate.
    fn write_tree(&self, s: &mut OsString) {
        self.write_tree_offset(0, s);
    }
    /// Translate `self` to a statement in command line language.
    fn to_stmt(&self) -> OsString {
        let mut s = OsString::new();
        self.write_stmt(&mut s);
        s
    }
    /// Translate `self` to a tree (pretty but verbose equivalent to a statement).
    fn to_tree(&self) -> OsString {
        let mut s = OsString::new();
        self.write_tree(&mut s);
        s
    }
    /// Translate `self` to command line arguments.
    fn to_args(&self) -> Vec<OsString> {
        let mut v = Vec::new();
        self.append_args(&mut v);
        v
    }
}

// public within the crate to allow `impl crate::translate::private::Sealed for ...`
// to be included as part of the `#[derive(Translate)]`.
pub(crate) mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    mod level {
        use super::*;

        #[derive(Translate)]
        struct Level0 {
            a: i32,
            c: Level1,
            b: f64,
        }

        #[derive(Translate)]
        #[declare = "level1"]
        struct Level1 {
            d: i32,
            e: OsString,
            f: Level2,
        }

        #[derive(Translate)]
        #[declare = "level2"]
        struct Level2 {
            g: i32,
            h: i32,
        }

        fn example() -> Level0 {
            Level0 {
                a: 1,
                b: 2.0,
                c: Level1 {
                    d: 4,
                    e: OsString::from("foo"),
                    f: Level2 { g: 5, h: 6 },
                },
            }
        }

        #[test]
        fn to_stmt() {
            let x = example();

            assert_eq!(x.c.f.to_stmt(), "level2 g=5 h=6");
            assert_eq!(x.c.to_stmt(), "level1 d=4 e=foo level2 g=5 h=6");
            assert_eq!(x.to_stmt(), "a=1 level1 d=4 e=foo level2 g=5 h=6 b=2");
        }

        #[test]
        fn to_tree() {
            let x = example();

            let rhs = "\
            level2
  g = 5
  h = 6";
            assert_eq!(x.c.f.to_tree(), rhs);

            let rhs = "\
            level1
  d = 4
  e = foo
  level2
    g = 5
    h = 6";
            assert_eq!(x.c.to_tree(), rhs);

            let rhs = "\
            a = 1
level1
  d = 4
  e = foo
  level2
    g = 5
    h = 6
b = 2";
            assert_eq!(x.to_tree(), rhs);
        }
    }

    mod actual {
        use super::*;
        use crate::argtree::*;
        use crate::method::*;
        // use crate::sample::*;
        // use crate::variational::*;

        fn join_with_ws(v: &[OsString]) -> OsString {
            let n = v.len();
            if n != 0 {
                let cap: usize = v.iter().map(|x| x.len()).sum();
                let mut s = OsString::with_capacity(cap + n - 1);
                let mut iter = v.into_iter();
                s.push(iter.next().unwrap());
                for x in iter {
                    s.push(" ");
                    s.push(x);
                }
                s
            } else {
                OsString::new()
            }
        }
        fn test_args_eq_stmt<T: Translate>(x: &T) {
            assert_eq!(x.to_stmt(), join_with_ws(&x.to_args()));
        }

        #[test]
        fn engine() {
            let e = Engine::Nuts { max_depth: 10 };
            assert_eq!(e.to_stmt(), "engine=nuts max_depth=10");
            let rhs = "\
engine = nuts
  nuts
    max_depth = 10";
            assert_eq!(e.to_tree(), rhs);
            test_args_eq_stmt(&e);
        }

        #[test]
        fn algorithm() {
            let a = SampleAlgorithm::Hmc {
                engine: Engine::Static { int_time: 2.5 },
                metric: Metric::DiagE,
                metric_file: "bar.csv".into(),
                stepsize: 1.0,
                stepsize_jitter: 0.0,
            };
            assert_eq!(a.to_stmt(), "algorithm=hmc engine=static int_time=2.5 metric=diag_e metric_file=bar.csv stepsize=1 stepsize_jitter=0");
            let rhs = "\
algorithm = hmc
  hmc
    engine = static
      static
        int_time = 2.5
    metric = diag_e
    metric_file = bar.csv
    stepsize = 1
    stepsize_jitter = 0";
            assert_eq!(a.to_tree(), rhs);
            test_args_eq_stmt(&a);

            let a = SampleAlgorithm::FixedParam;
            assert_eq!(a.to_stmt(), "algorithm=fixed_param");
            let rhs = "\
algorithm = fixed_param
  fixed_param";
            assert_eq!(a.to_tree(), rhs);
            test_args_eq_stmt(&a);
        }

        #[test]
        fn metric() {
            use Metric::*;

            for (m, s) in [(UnitE, "unit_e"), (DiagE, "diag_e"), (DenseE, "dense_e")] {
                assert_eq!(m.to_stmt(), format!("metric={}", s).as_str());
                assert_eq!(m.to_tree(), format!("metric = {}", s).as_str());
                test_args_eq_stmt(&m);
            }
        }

        #[test]
        fn adapt() {
            let sa = SampleAdapt {
                engaged: true,
                gamma: 0.05,
                delta: 0.8,
                kappa: 0.75,
                t0: 10.0,
                init_buffer: 75,
                term_buffer: 50,
                window: 25,
            };
            assert_eq!(sa.to_stmt(), "adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25");
            let rhs = "\
adapt
  engaged = 1
  gamma = 0.05
  delta = 0.8
  kappa = 0.75
  t0 = 10
  init_buffer = 75
  term_buffer = 50
  window = 25";
            assert_eq!(sa.to_tree(), rhs);
            test_args_eq_stmt(&sa);
        }

        #[test]
        fn method() {
            let sa = SampleAdapt {
                engaged: true,
                gamma: 0.05,
                delta: 0.8,
                kappa: 0.75,
                t0: 10.0,
                init_buffer: 75,
                term_buffer: 50,
                window: 25,
            };
            let m = Method::Sample {
                num_samples: 1000,
                num_warmup: 1000,
                save_warmup: true,
                thin: 1,
                adapt: sa,
                algorithm: SampleAlgorithm::Hmc {
                    engine: Engine::Nuts { max_depth: 10 },
                    metric: Metric::DiagE,
                    metric_file: "".into(),
                    stepsize: 1.0,
                    stepsize_jitter: 0.0,
                },
                num_chains: 4,
            };
            assert_eq!(m.to_stmt(), "method=sample num_samples=1000 num_warmup=1000 save_warmup=1 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=10 metric=diag_e metric_file= stepsize=1 stepsize_jitter=0 num_chains=4");
            let rhs = "\
method = sample
  sample
    num_samples = 1000
    num_warmup = 1000
    save_warmup = 1
    thin = 1
    adapt
      engaged = 1
      gamma = 0.05
      delta = 0.8
      kappa = 0.75
      t0 = 10
      init_buffer = 75
      term_buffer = 50
      window = 25
    algorithm = hmc
      hmc
        engine = nuts
          nuts
            max_depth = 10
        metric = diag_e
        metric_file = 
        stepsize = 1
        stepsize_jitter = 0
    num_chains = 4";
            assert_eq!(m.to_tree(), rhs);
            test_args_eq_stmt(&m);

            let m = Method::Variational {
                algorithm: VariationalAlgorithm::MeanField,
                iter: 10000,
                grad_samples: 1,
                elbo_samples: 100,
                eta: 1.0,
                adapt: VariationalAdapt {
                    engaged: true,
                    iter: 50,
                },
                tol_rel_obj: 0.01,
                eval_elbo: 100,
                output_samples: 100,
            };
            assert_eq!(m.to_stmt(), "method=variational algorithm=meanfield iter=10000 grad_samples=1 elbo_samples=100 eta=1 adapt engaged=1 iter=50 tol_rel_obj=0.01 eval_elbo=100 output_samples=100");
            let rhs = "\
method = variational
  variational
    algorithm = meanfield
      meanfield
    iter = 10000
    grad_samples = 1
    elbo_samples = 100
    eta = 1
    adapt
      engaged = 1
      iter = 50
    tol_rel_obj = 0.01
    eval_elbo = 100
    output_samples = 100";
            assert_eq!(m.to_tree(), rhs);
            test_args_eq_stmt(&m);
        }

        #[test]
        fn argtree() {
            let sa = SampleAdapt {
                engaged: true,
                gamma: 0.05,
                delta: 0.8,
                kappa: 0.75,
                t0: 10.0,
                init_buffer: 75,
                term_buffer: 50,
                window: 25,
            };

            let m = Method::Sample {
                num_samples: 1000,
                num_warmup: 1000,
                save_warmup: true,
                thin: 1,
                adapt: sa,
                algorithm: SampleAlgorithm::Hmc {
                    engine: Engine::Nuts { max_depth: 10 },
                    metric: Metric::DiagE,
                    metric_file: "".into(),
                    stepsize: 1.0,
                    stepsize_jitter: 0.0,
                },
                num_chains: 4,
            };

            let t = ArgTree {
                method: m,
                id: 1,
                init: "2".into(),
                num_threads: 12,
                data: Data {
                    file: "bernoulli.data.json".into(),
                },
                random: Random { seed: 123456789 },
                output: Output {
                    file: "output.csv".into(),
                    diagnostic_file: "".into(),
                    profile_file: "profile.csv".into(),
                    refresh: 100,
                    sig_figs: 18,
                },
            };

            assert_eq!(t.to_stmt(), "method=sample num_samples=1000 num_warmup=1000 save_warmup=1 thin=1 adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25 algorithm=hmc engine=nuts max_depth=10 metric=diag_e metric_file= stepsize=1 stepsize_jitter=0 num_chains=4 id=1 data file=bernoulli.data.json init=2 random seed=123456789 output file=output.csv diagnostic_file= refresh=100 sig_figs=18 profile_file=profile.csv num_threads=12");

            let rhs = "\
method = sample
  sample
    num_samples = 1000
    num_warmup = 1000
    save_warmup = 1
    thin = 1
    adapt
      engaged = 1
      gamma = 0.05
      delta = 0.8
      kappa = 0.75
      t0 = 10
      init_buffer = 75
      term_buffer = 50
      window = 25
    algorithm = hmc
      hmc
        engine = nuts
          nuts
            max_depth = 10
        metric = diag_e
        metric_file = 
        stepsize = 1
        stepsize_jitter = 0
    num_chains = 4
id = 1
data
  file = bernoulli.data.json
init = 2
random
  seed = 123456789
output
  file = output.csv
  diagnostic_file = 
  refresh = 100
  sig_figs = 18
  profile_file = profile.csv
num_threads = 12";
            assert_eq!(t.to_tree(), rhs);
            test_args_eq_stmt(&t);

            let m = Method::Variational {
                algorithm: VariationalAlgorithm::MeanField,
                iter: 10000,
                grad_samples: 1,
                elbo_samples: 100,
                eta: 1.0,
                adapt: VariationalAdapt {
                    engaged: true,
                    iter: 50,
                },
                tol_rel_obj: 0.01,
                eval_elbo: 100,
                output_samples: 100,
            };
            let mut t = t;
            t.method = m;
            assert_eq!(t.to_stmt(), "method=variational algorithm=meanfield iter=10000 grad_samples=1 elbo_samples=100 eta=1 adapt engaged=1 iter=50 tol_rel_obj=0.01 eval_elbo=100 output_samples=100 id=1 data file=bernoulli.data.json init=2 random seed=123456789 output file=output.csv diagnostic_file= refresh=100 sig_figs=18 profile_file=profile.csv num_threads=12");

            let rhs = "\
method = variational
  variational
    algorithm = meanfield
      meanfield
    iter = 10000
    grad_samples = 1
    elbo_samples = 100
    eta = 1
    adapt
      engaged = 1
      iter = 50
    tol_rel_obj = 0.01
    eval_elbo = 100
    output_samples = 100
id = 1
data
  file = bernoulli.data.json
init = 2
random
  seed = 123456789
output
  file = output.csv
  diagnostic_file = 
  refresh = 100
  sig_figs = 18
  profile_file = profile.csv
num_threads = 12";
            assert_eq!(t.to_tree(), rhs);
            test_args_eq_stmt(&t);
        }
    }
}
