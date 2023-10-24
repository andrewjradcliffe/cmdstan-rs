use crate::method::Method;
use std::fmt::Write;

#[derive(Debug)]
pub struct SampleBuilder {
    num_samples: Option<i32>,
    num_warmup: Option<i32>,
    save_warmup: Option<bool>,
    thin: Option<i32>,
    adapt: Option<SampleAdapt>,
    algorithm: Option<SampleAlgorithm>,
    num_chains: Option<i32>,
}
macro_rules! insert_field {
    ($F:ident, $T:ident) => {
        pub fn $F(self, $F: $T) -> Self {
            let mut me = self;
            let _ = me.$F.insert($F);
            me
        }
    };
}
impl SampleBuilder {
    pub fn builder() -> Self {
        Self {
            num_samples: None,
            num_warmup: None,
            save_warmup: None,
            thin: None,
            adapt: None,
            algorithm: None,
            num_chains: None,
        }
    }
    insert_field!(num_samples, i32);
    insert_field!(num_warmup, i32);
    insert_field!(save_warmup, bool);
    insert_field!(thin, i32);
    insert_field!(adapt, SampleAdapt);
    insert_field!(algorithm, SampleAlgorithm);
    insert_field!(num_chains, i32);
    pub fn build(self) -> Method {
        let num_samples = self.num_samples.unwrap_or(1000);
        let num_warmup = self.num_warmup.unwrap_or(1000);
        let save_warmup = self.save_warmup.unwrap_or(false);
        let thin = self.thin.unwrap_or(1);
        let adapt = self.adapt.unwrap_or_default();
        let algorithm = self.algorithm.unwrap_or_default();
        let num_chains = self.num_chains.unwrap_or(1);
        Method::Sample {
            num_samples,
            num_warmup,
            save_warmup,
            thin,
            adapt,
            algorithm,
            num_chains,
        }
    }
}

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
    // Rather than define the defaults in two places, the `build` method of SampleAdaptBuilder,
    // called on an all-None builder, should serve as the single source of truth.
    fn default() -> Self {
        SampleAdaptBuilder::builder().build()
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
    pub fn builder() -> SampleAdaptBuilder {
        SampleAdaptBuilder::builder()
    }
}

#[derive(Debug)]
pub struct SampleAdaptBuilder {
    engaged: Option<bool>,
    gamma: Option<f64>,
    delta: Option<f64>,
    kappa: Option<f64>,
    t0: Option<f64>,
    init_buffer: Option<u32>,
    term_buffer: Option<u32>,
    window: Option<u32>,
}
impl SampleAdaptBuilder {
    pub fn builder() -> Self {
        Self {
            engaged: None,
            gamma: None,
            delta: None,
            kappa: None,
            t0: None,
            init_buffer: None,
            term_buffer: None,
            window: None,
        }
    }
    insert_field!(engaged, bool);
    insert_field!(gamma, f64);
    insert_field!(delta, f64);
    insert_field!(kappa, f64);
    insert_field!(t0, f64);
    insert_field!(init_buffer, u32);
    insert_field!(term_buffer, u32);
    insert_field!(window, u32);
    pub fn build(self) -> SampleAdapt {
        let engaged = self.engaged.unwrap_or(true);
        let gamma = self.gamma.unwrap_or(0.05);
        let delta = self.delta.unwrap_or(0.8);
        let kappa = self.kappa.unwrap_or(0.75);
        let t0 = self.t0.unwrap_or(10.0);
        let init_buffer = self.init_buffer.unwrap_or(75);
        let term_buffer = self.term_buffer.unwrap_or(50);
        let window = self.window.unwrap_or(25);
        SampleAdapt {
            engaged,
            gamma,
            delta,
            kappa,
            t0,
            init_buffer,
            term_buffer,
            window,
        }
    }
}

pub struct HmcBuilder {
    engine: Option<Engine>,
    metric: Option<Metric>,
    metric_file: Option<String>,
    stepsize: Option<f64>,
    stepsize_jitter: Option<f64>,
}

impl HmcBuilder {
    pub fn builder() -> Self {
        Self {
            engine: None,
            metric: None,
            metric_file: None,
            stepsize: None,
            stepsize_jitter: None,
        }
    }

    insert_field!(engine, Engine);
    insert_field!(metric, Metric);
    insert_field!(metric_file, String);
    insert_field!(stepsize, f64);
    insert_field!(stepsize_jitter, f64);

    pub fn build(self) -> SampleAlgorithm {
        let engine = self.engine.unwrap_or_default();
        let metric = self.metric.unwrap_or_default();
        let metric_file = self.metric_file.unwrap_or_else(|| "".to_string());
        let stepsize = self.stepsize.unwrap_or(1.0);
        let stepsize_jitter = self.stepsize_jitter.unwrap_or(0.0);
        SampleAlgorithm::Hmc {
            engine,
            metric,
            metric_file,
            stepsize,
            stepsize_jitter,
        }
    }
}

/// Sampling algorithm
/// Valid values: hmc, fixed_param
/// Defaults to hmc
#[derive(Debug, PartialEq)]
pub enum SampleAlgorithm {
    /// Hamiltonian Monte Carlo
    Hmc {
        /// Engine for Hamiltonian Monte Carlo
        /// Valid values: static, nuts
        /// Defaults to nuts
        engine: Engine,
        /// Geometry of base manifold
        /// Valid values: unit_e, diag_e, dense_e
        /// Defaults to diag_e
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
        Self::from(HmcBuilder::builder())
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
impl From<HmcBuilder> for SampleAlgorithm {
    fn from(hmc: HmcBuilder) -> SampleAlgorithm {
        hmc.build()
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
        Self::from(NutsBuilder::builder())
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
impl From<StaticBuilder> for Engine {
    fn from(x: StaticBuilder) -> Self {
        x.build()
    }
}
impl From<NutsBuilder> for Engine {
    fn from(x: NutsBuilder) -> Self {
        x.build()
    }
}

/// Static integration time
#[derive(Debug, PartialEq)]
pub struct StaticBuilder {
    int_time: Option<f64>,
}
impl StaticBuilder {
    pub fn builder() -> StaticBuilder {
        StaticBuilder { int_time: None }
    }
    insert_field!(int_time, f64);
    pub fn build(self) -> Engine {
        let int_time = self.int_time.unwrap_or(std::f64::consts::TAU);
        Engine::Static { int_time }
    }
}

/// The No-U-Turn Sampler
#[derive(Debug, PartialEq)]
pub struct NutsBuilder {
    max_depth: Option<i32>,
}
impl NutsBuilder {
    pub fn builder() -> NutsBuilder {
        NutsBuilder { max_depth: None }
    }
    insert_field!(max_depth, i32);
    pub fn build(self) -> Engine {
        let max_depth = self.max_depth.unwrap_or(10);
        Engine::Nuts { max_depth }
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
    mod sample {
        use super::*;

        #[test]
        fn builder() {
            let x = SampleBuilder::builder();
            let y = x.num_samples(2);
            let z = y.num_warmup(2);
            assert_eq!(z.num_samples, Some(2));
            assert_eq!(z.num_warmup, Some(2));

            let z = SampleBuilder::builder()
                .num_samples(2)
                .num_warmup(2)
                .num_samples(10);
            assert_eq!(z.num_samples, Some(10));
            assert_eq!(z.num_warmup, Some(2));

            let x = SampleBuilder::builder()
                .num_samples(2)
                .num_warmup(2)
                .save_warmup(true)
                .thin(5);
            assert_eq!(x.save_warmup, Some(true));
            assert_eq!(x.thin, Some(5));

            let x = SampleBuilder::builder()
                .algorithm(SampleAlgorithm::default())
                .adapt(SampleAdapt::default());
            assert_eq!(x.adapt, Some(SampleAdapt::default()));
            assert_eq!(x.algorithm, Some(SampleAlgorithm::default()));

            let x = SampleBuilder::builder()
                .num_samples(1)
                .num_warmup(2)
                .save_warmup(true)
                .thin(5)
                .num_chains(10);
            assert_eq!(x.num_chains, Some(10));
        }
    }

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
                .init_buffer(1)
                .term_buffer(2)
                .window(3)
                .build();
            assert_eq!(x.engaged, false);
            assert_eq!(x.gamma, 0.1);
            assert_eq!(x.delta, 0.2);
            assert_eq!(x.kappa, 0.3);
            assert_eq!(x.t0, 0.4);
            assert_eq!(x.init_buffer, 1);
            assert_eq!(x.term_buffer, 2);
            assert_eq!(x.window, 3);
        }

        #[test]
        fn command_fragment() {
            let x = SampleAdapt::default();
            assert_eq!(x.command_fragment(), "adapt engaged=1 gamma=0.05 delta=0.8 kappa=0.75 t0=10 init_buffer=75 term_buffer=50 window=25");
        }
    }

    #[cfg(test)]
    mod algorithm {
        use super::*;

        #[test]
        fn command_fragment() {
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
        fn default() {
            let x = StaticBuilder::builder().build();
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
        fn command_fragment() {
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

        #[test]
        fn from() {
            let x = Engine::from(StaticBuilder::builder());
            assert!(matches!(x, Engine::Static { int_time: _ }));

            let x = Engine::from(NutsBuilder::builder().max_depth(5));
            assert!(matches!(x, Engine::Nuts { max_depth: 5 }));
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
        fn command_fragment() {
            let x = Metric::default();
            assert_eq!(x.command_fragment(), "metric=diag_e");
        }
    }
}
