use std::fmt::Write;

/// Warmup Adaptation
#[derive(Debug, PartialEq)]
pub struct SampleAdapt {
    /// Adaptation engaged?
    /// Valid values: [0, 1]
    /// Defaults to 1
    engaged: bool,
    /// Adaptation regularization scale
    /// Valid values: 0 < gamma
    /// Defaults to 0.05
    gamma: f64,
    /// Adaptation target acceptance statistic
    /// Valid values: 0 < delta < 1
    /// Defaults to 0.8
    delta: f64,
    /// Adaptation relaxation exponent
    /// Valid values: 0 < kappa
    /// Defaults to 0.75
    kappa: f64,
    /// Adaptation iteration offset
    /// Valid values: 0 < t0
    /// Defaults to 10
    t0: f64,
    /// Width of initial fast adaptation interval
    /// Valid values: All
    /// Defaults to 75
    init_buffer: u32,
    /// Width of final fast adaptation interval
    /// Valid values: All
    /// Defaults to 50
    term_buffer: u32,
    /// Initial width of slow adaptation interval
    /// Valid values: All
    /// Defaults to 25
    window: u32,
}
impl Default for SampleAdapt {
    fn default() -> Self {
        Self {
            engaged: true,
            gamma: 0.05,
            delta: 0.8,
            kappa: 0.75,
            t0: 10.0,
            init_buffer: 75,
            term_buffer: 50,
            window: 25,
        }
    }
}

impl SampleAdapt {
    pub fn command_fragment(&self) -> String {
        let mut s = String::from("adapt");
        write!(&mut s, " engaged={}", self.engaged as u8).unwrap();
        write!(&mut s, " gamma={}", self.gamma).unwrap();
        write!(&mut s, " delta={}", self.delta).unwrap();
        write!(&mut s, " kappa={}", self.kappa).unwrap();
        write!(&mut s, " t0={}", self.t0).unwrap();
        write!(&mut s, " init_buffer={}", self.init_buffer).unwrap();
        write!(&mut s, " term_buffer={}", self.term_buffer).unwrap();
        write!(&mut s, " window={}", self.window).unwrap();
        s
    }
}

/// Sampling algorithm
/// Valid values: hmc, fixed_param
/// Defaults to hmc
#[derive(Debug, PartialEq)]
pub enum SampleAlgorithm {
    /// Hamiltonian Monte Carlo
    Hmc {
        engine: Engine,
        metric: Metric,
        /// Input file with precomputed Euclidean metric
        /// Valid values: Path to existing file
        /// Defaults to ""
        metric_file: String,
        /// Step size for discrete evolution
        /// Valid values: 0 < stepsize
        /// Defaults to 1
        stepsize: f64,
        /// Uniformly random jitter of the stepsize, in percent
        /// Valid values: 0 <= stepsize_jitter <= 1
        /// Defaults to 0
        stepsize_jitter: f64,
    },
    /// Fixed Parameter Sampler
    FixedParam,
}

impl Default for SampleAlgorithm {
    fn default() -> Self {
        SampleAlgorithm::Hmc {
            engine: Engine::default(),
            metric: Metric::default(),
            metric_file: String::from(""),
            stepsize: 1.0,
            stepsize_jitter: 0.0,
        }
    }
}

impl SampleAlgorithm {
    pub fn command_fragment(&self) -> String {
        match &self {
            Self::Hmc {
                engine,
                metric,
                metric_file,
                stepsize,
                stepsize_jitter,
            } => {
                let mut s = String::from("algorithm=hmc");
                write!(&mut s, " {}", engine.command_fragment()).unwrap();
                write!(&mut s, " {}", metric.command_fragment()).unwrap();
                match metric_file.as_ref() {
                    "" => (),
                    x => write!(&mut s, " metric_file={}", x).unwrap(),
                }
                write!(&mut s, " stepsize={}", stepsize).unwrap();
                write!(&mut s, " stepsize_jitter={}", stepsize_jitter).unwrap();
                s
            }
            Self::FixedParam => "algorithm=fixed_param".to_string(),
        }
    }
}

/// Engine for Hamiltonian Monte Carlo
/// Valid values: static, nuts
/// Defaults to nuts
#[derive(Debug, PartialEq)]
pub enum Engine {
    /// Static integration time
    Static {
        /// Total integration time for Hamiltonian evolution
        /// Valid values: 0 < int_time
        /// Defaults to 2 * pi
        int_time: f64,
    },
    /// The No-U-Turn Sampler
    Nuts {
        /// Maximum tree depth
        /// Valid values: 0 < max_depth
        /// Defaults to 10
        max_depth: i32,
    },
}
impl Default for Engine {
    fn default() -> Self {
        Engine::Nuts { max_depth: 10 }
    }
}

impl Engine {
    pub fn command_fragment(&self) -> String {
        match &self {
            Engine::Nuts { max_depth } => {
                format!("engine=nuts max_depth={}", max_depth)
            }
            Engine::Static { int_time } => {
                format!("engine=static int_time={}", int_time)
            }
        }
    }
}

/// Geometry of base manifold
/// Valid values: unit_e, diag_e, dense_e
/// Defaults to diag_e
#[derive(Debug, PartialEq, Default)]
pub enum Metric {
    /// Euclidean manifold with unit metric
    UnitE,
    /// Euclidean manifold with diag metric
    #[default]
    DiagE,
    /// Euclidean manifold with dense metric
    DenseE,
}

impl Metric {
    pub fn command_fragment(&self) -> String {
        match &self {
            Metric::UnitE => "metric=unit_e",
            Metric::DiagE => "metric=diag_e",
            Metric::DenseE => "metric=dense_e",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod adapt {
        use super::*;

        #[test]
        fn default_works() {
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
        fn command_fragment_works() {
            let x = SampleAdapt::default();
            assert_eq!(x.command_fragment(), "adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25");
        }
    }

    #[cfg(test)]
    mod algorithm {
        use super::*;

        #[test]
        fn command_fragment_works() {
            let mut x = SampleAlgorithm::default();
            assert_eq!(
                x.command_fragment(),
                "algorithm=hmc engine=nuts max_depth=10 metric=diag_e stepsize=1 stepsize_jitter=0"
            );
            match x {
                SampleAlgorithm::Hmc {
                    ref mut metric_file,
                    ..
                } => {
                    metric_file.push_str("my_metric.json");
                }
                _ => (),
            };
            assert_eq!(
                x.command_fragment(),
                "algorithm=hmc engine=nuts max_depth=10 metric=diag_e metric_file=my_metric.json stepsize=1 stepsize_jitter=0"
            );
        }
    }

    #[cfg(test)]
    mod engine {
        use super::*;

        #[test]
        fn default_works() {
            let x = Engine::default();
            assert_eq!(x, Engine::Nuts { max_depth: 10 });
        }

        #[test]
        fn command_fragment_works() {
            let x = Engine::default();
            assert_eq!(x.command_fragment(), "engine=nuts max_depth=10");

            let x = Engine::Static {
                int_time: std::f64::consts::TAU,
            };
            assert_eq!(
                x.command_fragment(),
                format!("engine=static int_time={}", std::f64::consts::TAU)
            );
        }
    }

    #[cfg(test)]
    mod metric {
        use super::*;

        #[test]
        fn default_works() {
            let x = Metric::default();
            assert_eq!(x, Metric::DiagE);
        }

        #[test]
        fn command_fragment_works() {
            let x = Metric::default();
            assert_eq!(x.command_fragment(), "metric=diag_e");
        }
    }
}
