use crate::builder::Builder;
use crate::translate::Translate;
use std::ffi::OsString;

/// Warmup Adaptation for [`Method::Sample`][crate::method::Method::Sample]
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "adapt"]
pub struct SampleAdapt {
    /// Adaptation engaged?
    /// Defaults to `true`.
    ///
    /// At command line, this presents as `false` => 0, `true` => 1,
    /// with valid values 0 or 1.
    #[defaults_to = true]
    pub engaged: bool,
    /// Adaptation regularization scale.
    /// Valid values: `0 < gamma`.
    /// Defaults to `0.05`.
    #[defaults_to = 0.05]
    pub gamma: f64,
    /// Adaptation target acceptance statistic.
    /// Valid values: `0 < delta < 1`
    /// Defaults to `0.8`.
    #[defaults_to = 0.8]
    pub delta: f64,
    /// Adaptation relaxation exponent.
    /// Valid values: `0 < kappa`.
    /// Defaults to `0.75`.
    #[defaults_to = 0.75]
    pub kappa: f64,
    /// Adaptation iteration offset.
    /// Valid values: `0 < t0`
    /// Defaults to `10.0`.
    #[defaults_to = 10.0]
    pub t0: f64,
    /// Width of initial fast adaptation interval.
    /// Valid values: All.
    /// Defaults to `75`.
    #[defaults_to = 75]
    pub init_buffer: u32,
    /// Width of final fast adaptation interval.
    /// Valid values: All.
    /// Defaults to `50`.
    #[defaults_to = 50]
    pub term_buffer: u32,
    /// Initial width of slow adaptation interval.
    /// Valid values: All.
    /// Defaults to `25`.
    #[defaults_to = 25]
    pub window: u32,
}

/// Sampling algorithm. Defaults to [`SampleAlgorithm::Hmc`].
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "algorithm"]
pub enum SampleAlgorithm {
    /// Hamiltonian Monte Carlo.
    /// To construct, use [`HmcBuilder`].
    #[non_exhaustive]
    Hmc {
        /// Engine for Hamiltonian Monte Carlo.
        /// Valid values: any variant of `Engine`.
        /// Defaults to [`Engine::Nuts`] (with respective defaults).
        engine: Engine,
        /// Geometry of base manifold.
        /// Valid values: any variant of `Metric`.
        /// Defaults to [`Metric::DiagE`].
        metric: Metric,
        /// Input file with precomputed Euclidean metric.
        /// Valid values: Path to existing file.
        /// Defaults to `""`.
        #[defaults_to = ""]
        metric_file: OsString,
        /// Step size for discrete evolution.
        /// Valid values: `0 < stepsize`.
        /// Defaults to `1`.
        #[defaults_to = 1.0]
        stepsize: f64,
        /// Uniformly random jitter of the stepsize, in percent.
        /// Valid values: `0 <= stepsize_jitter <= 1`
        /// Defaults to `0`.
        #[defaults_to = 0.0]
        stepsize_jitter: f64,
    },
    /// Fixed Parameter Sampler
    #[declare = "fixed_param"]
    FixedParam,
}

impl Default for SampleAlgorithm {
    fn default() -> Self {
        Self::from(HmcBuilder::new())
    }
}

/// Engine for Hamiltonian Monte Carlo. Defaults to [`Engine::Nuts`].
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "engine"]
pub enum Engine {
    /// Static integration time.
    /// To construct, use [`StaticBuilder`].
    #[non_exhaustive]
    Static {
        /// Total integration time for Hamiltonian evolution.
        /// Valid values: `0 < int_time`.
        /// Defaults to `2 * pi`.
        #[defaults_to = "std::f64::consts::TAU"]
        int_time: f64,
    },
    /// The No-U-Turn Sampler.
    /// To construct, use [`NutsBuilder`].
    #[non_exhaustive]
    Nuts {
        /// Maximum tree depth.
        /// Valid values: `0 < max_depth`.
        /// Defaults to `10`.
        #[defaults_to = 10]
        max_depth: i32,
    },
}
impl Default for Engine {
    fn default() -> Self {
        Self::from(NutsBuilder::new())
    }
}

/// Geometry of base manifold. Defaults to [`Metric::DiagE`]
#[derive(Debug, PartialEq, Default, Clone, Translate)]
#[declare = "metric"]
pub enum Metric {
    /// Euclidean manifold with unit metric
    #[declare = "unit_e"]
    UnitE,
    /// Euclidean manifold with diagonal metric
    #[default]
    #[declare = "diag_e"]
    DiagE,
    /// Euclidean manifold with dense metric
    #[declare = "dense_e"]
    DenseE,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod adapt {
        use super::*;

        #[test]
        fn default() {
            let x = SampleAdapt::default();
            assert_eq!(
                x,
                SampleAdapt {
                    engaged: true,
                    gamma: 0.05,
                    delta: 0.8,
                    kappa: 0.75,
                    t0: 10.0,
                    init_buffer: 75,
                    term_buffer: 50,
                    window: 25,
                }
            );
        }

        #[test]
        fn builder() {
            let x = SampleAdapt::builder()
                .engaged(false)
                .gamma(0.1)
                .delta(0.2)
                .kappa(0.3)
                .t0(0.4)
                .init_buffer(1u32)
                .term_buffer(2u32)
                .window(3u32)
                .build();
            assert!(!x.engaged);
            assert_eq!(x.gamma, 0.1);
            assert_eq!(x.delta, 0.2);
            assert_eq!(x.kappa, 0.3);
            assert_eq!(x.t0, 0.4);
            assert_eq!(x.init_buffer, 1);
            assert_eq!(x.term_buffer, 2);
            assert_eq!(x.window, 3);
        }

        #[test]
        fn to_args() {
            let x = SampleAdapt::default();
            assert_eq!(
                x.to_args(),
                vec![
                    "adapt",
                    "engaged=1",
                    "gamma=0.05",
                    "delta=0.8",
                    "kappa=0.75",
                    "t0=10",
                    "init_buffer=75",
                    "term_buffer=50",
                    "window=25",
                ]
            );

            let x = SampleAdapt::builder()
                .engaged(false)
                .gamma(0.1)
                .delta(0.2)
                .kappa(0.3)
                .t0(0.4)
                .init_buffer(1u32)
                .term_buffer(2u32)
                .window(3u32)
                .build();
            assert_eq!(
                x.to_args(),
                vec![
                    "adapt",
                    "engaged=0",
                    "gamma=0.1",
                    "delta=0.2",
                    "kappa=0.3",
                    "t0=0.4",
                    "init_buffer=1",
                    "term_buffer=2",
                    "window=3",
                ]
            );
        }
    }

    #[cfg(test)]
    mod algorithm {
        use super::*;

        #[test]
        fn builder() {
            let x = HmcBuilder::new()
                .engine(Engine::Static { int_time: 2.5 })
                .metric(Metric::DenseE)
                .metric_file("big.txt")
                .stepsize(10.0)
                .stepsize_jitter(0.5)
                .build();
            let SampleAlgorithm::Hmc {
                engine,
                metric,
                metric_file,
                stepsize,
                stepsize_jitter,
            } = x
            else {
                unreachable!();
            };
            assert_eq!(engine, Engine::Static { int_time: 2.5 });
            assert_eq!(metric, Metric::DenseE);
            assert_eq!(metric_file, "big.txt");
            assert_eq!(stepsize, 10.0);
            assert_eq!(stepsize_jitter, 0.5);

            let x = HmcBuilder::new()
                .engine(NutsBuilder::new().max_depth(100).build())
                .build();
            let SampleAlgorithm::Hmc { engine, .. } = x else {
                unreachable!();
            };
            assert_eq!(engine, Engine::Nuts { max_depth: 100 });
        }

        #[test]
        fn from() {
            let x = HmcBuilder::new();
            assert_eq!(SampleAlgorithm::from(x), HmcBuilder::new().build());
        }

        #[test]
        fn to_args() {
            let mut x = HmcBuilder::new().build();
            assert_eq!(
                x.to_args(),
                vec![
                    "algorithm=hmc",
                    "engine=nuts",
                    "max_depth=10",
                    "metric=diag_e",
                    "metric_file=",
                    "stepsize=1",
                    "stepsize_jitter=0",
                ]
            );
            let SampleAlgorithm::Hmc {
                ref mut metric_file,
                ..
            } = x
            else {
                unreachable!()
            };
            metric_file.push("my_metric.json");
            assert_eq!(
                x.to_args(),
                vec![
                    "algorithm=hmc",
                    "engine=nuts",
                    "max_depth=10",
                    "metric=diag_e",
                    "metric_file=my_metric.json",
                    "stepsize=1",
                    "stepsize_jitter=0",
                ]
            );

            let x = HmcBuilder::new()
                .engine(Engine::Static { int_time: 2.5 })
                .metric(Metric::DenseE)
                .metric_file("big.txt")
                .stepsize(10.0)
                .stepsize_jitter(0.5)
                .build();

            assert_eq!(
                x.to_args(),
                vec![
                    "algorithm=hmc",
                    "engine=static",
                    "int_time=2.5",
                    "metric=dense_e",
                    "metric_file=big.txt",
                    "stepsize=10",
                    "stepsize_jitter=0.5",
                ]
            );
            let x = SampleAlgorithm::FixedParam;
            assert_eq!(x.to_args(), vec!["algorithm=fixed_param"]);
        }
    }

    #[cfg(test)]
    mod engine {
        use super::*;

        #[test]
        fn builder() {
            let x = StaticBuilder::new().int_time(2.5).build();
            assert_eq!(x, Engine::Static { int_time: 2.5 });

            let x = NutsBuilder::new().max_depth(100).build();
            assert_eq!(x, Engine::Nuts { max_depth: 100 });
        }

        #[test]
        fn default() {
            let x = StaticBuilder::new().build();
            assert_eq!(
                x,
                Engine::Static {
                    int_time: std::f64::consts::TAU
                }
            );
            let x = Engine::default();
            assert_eq!(x, Engine::Nuts { max_depth: 10 });
        }

        #[test]
        fn from() {
            let x = Engine::from(StaticBuilder::new().int_time(2.5));
            assert_eq!(x, Engine::Static { int_time: 2.5 });

            let x = Engine::from(NutsBuilder::new().max_depth(5));
            assert_eq!(x, Engine::Nuts { max_depth: 5 });
        }

        #[test]
        fn to_args() {
            let x = Engine::default();
            assert_eq!(x.to_args(), vec!["engine=nuts", "max_depth=10"]);

            let x = Engine::Static {
                int_time: std::f64::consts::TAU,
            };
            assert_eq!(
                x.to_args(),
                vec![
                    "engine=static",
                    format!("int_time={}", std::f64::consts::TAU).as_str()
                ]
            );
        }
    }

    #[cfg(test)]
    mod metric {
        use super::*;

        #[test]
        fn default() {
            let x = Metric::default();
            assert_eq!(x, Metric::DiagE);
        }

        #[test]
        fn to_args() {
            let x = Metric::UnitE;
            assert_eq!(x.to_args(), vec!["metric=unit_e"]);
            let x = Metric::DiagE;
            assert_eq!(x.to_args(), vec!["metric=diag_e"]);
            let x = Metric::DenseE;
            assert_eq!(x.to_args(), vec!["metric=dense_e"]);
        }
    }
}
