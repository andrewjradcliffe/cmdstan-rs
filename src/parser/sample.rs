use crate::method::Method;
use crate::parser::*;
use crate::sample::*;

impl FromStr for Metric {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::metric_as_type, s) {
            Ok(mut pair) => {
                let pair = pair.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(MetricError(format!("{e:#?}"))),
        }
    }
}

impl Metric {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::metric => {
                let variant = match pair.into_inner().next() {
                    Some(pair) => Self::classify_prechecked(pair),
                    _ => Self::default(),
                };
                Ok(variant)
            }
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }

    // This should only be used in pre-checked contexts, else it will
    // panic. That is, it should only be used on the inner pair of a
    // `Rule::metric`.
    #[inline]
    fn classify_prechecked(pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::unit_e => Metric::UnitE,
            Rule::diag_e => Metric::DiagE,
            Rule::dense_e => Metric::DenseE,
            _ => unreachable!(),
        }
    }
}

impl FromStr for Engine {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::engine_as_type, s) {
            Ok(mut pair) => {
                let pair = pair.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(EngineError(format!("{e:#?}"))),
        }
    }
}

// This enables us to skip parsing of n-1 floats.
// It is equivalent to parsing each and simply taking the last
// due to the fact that any number can be represented
// as a float.
// As the name suggests, this applies only to the Rule::r#static
// `Pair` which produces 0 or more Rule::int_time `Pair`s
fn unify_int_time(pair: Pair<'_, Rule>) -> Option<f64> {
    pair.into_inner()
        .filter_map(|p| p.into_inner().next())
        .last()
        .map(|p| p.as_str().parse::<f64>().unwrap())
}
// It would be nice to skip parsing of n-1 integers, but we
// have no other way to check that each value is < 2^31
fn unify_max_depth(pair: Pair<'_, Rule>) -> Result<Option<i32>, ParseGrammarError> {
    let pairs = pair.into_inner().filter_map(|p| p.into_inner().next());
    let mut max_depth: Option<i32> = None;
    for pair in pairs {
        match pair.as_str().parse::<i32>() {
            Ok(value) => {
                max_depth = Some(value);
            }
            Err(e) => return Err(EngineError(format!("{e:#?}"))),
        }
    }
    Ok(max_depth)
}

impl Engine {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::engine => {
                let variant = match pair.into_inner().next() {
                    Some(pair) => match pair.as_rule() {
                        Rule::nuts => {
                            let mut builder = NutsBuilder::new();
                            if let Some(value) = unify_max_depth(pair)? {
                                builder = builder.max_depth(value);
                            }
                            builder.build()
                        }
                        Rule::r#static => {
                            let mut builder = StaticBuilder::new();
                            if let Some(value) = unify_int_time(pair) {
                                builder = builder.int_time(value);
                            }
                            builder.build()
                        }
                        _ => unreachable!(),
                    },
                    _ => Self::default(),
                };
                Ok(variant)
            }
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }
}

macro_rules! number_arm {
    ($B:ident, $P:ident, $F:ident, $T:ty) => {
        match $P.into_inner().next() {
            Some(pair) => {
                let value = pair.as_str().parse::<$T>().unwrap();
                $B = $B.$F(value);
            }
            _ => (),
        }
    };
}
macro_rules! boolean_arm {
    ($B:ident, $P:ident, $F:ident) => {
        match $P.into_inner().next() {
            Some(pair) => {
                let value = match pair.as_rule() {
                    Rule::r#true => true,
                    Rule::r#false => false,
                    _ => unreachable!(),
                };
                $B = $B.$F(value);
            }
            _ => (),
        }
    };
}
macro_rules! path_arm {
    ($B:ident, $P:ident, $F:ident) => {
        match $P.into_inner().next() {
            Some(pair) => {
                $B = $B.$F(pair.as_str());
            }
            _ => (),
        }
    };
}

macro_rules! unify_sample_adapt_terms {
    ($B:ident, $sample_adapt:ident) => {
        let pairs = $sample_adapt
            .into_inner()
            .map(|sample_adapt_pair| sample_adapt_pair.into_inner().next().unwrap());
        for pair in pairs {
            match pair.as_rule() {
                Rule::engaged => boolean_arm!($B, pair, engaged),
                Rule::gamma => number_arm!($B, pair, gamma, f64),
                Rule::delta => number_arm!($B, pair, delta, f64),
                Rule::kappa => number_arm!($B, pair, kappa, f64),
                Rule::t0 => number_arm!($B, pair, t0, f64),
                Rule::init_buffer => {
                    number_arm!($B, pair, init_buffer, u32)
                }
                Rule::term_buffer => {
                    number_arm!($B, pair, term_buffer, u32)
                }
                Rule::window => number_arm!($B, pair, window, u32),
                _ => unreachable!(),
            }
        }
    };
}

impl SampleAdapt {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::sample_adapt => {
                let mut builder = Self::builder();
                unify_sample_adapt_terms!(builder, pair);
                Ok(builder.build())
            }
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }
}

impl FromStr for SampleAdapt {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::sample_adapt_as_type, s) {
            Ok(mut pair) => {
                let pair = pair.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(SampleAdaptError(format!("{e:#?}"))),
        }
    }
}

macro_rules! unify_hmc_terms {
    ($B:ident, $hmc:ident, $state:ident, $max_depth:ident, $int_time:ident) => {
        let pairs = $hmc
            .into_inner()
            .map(|hmc_term| hmc_term.into_inner().next().unwrap());
        for pair in pairs {
            match pair.as_rule() {
                Rule::stepsize => number_arm!($B, pair, stepsize, f64),
                Rule::stepsize_jitter => {
                    number_arm!($B, pair, stepsize_jitter, f64)
                }
                Rule::metric_file => path_arm!($B, pair, metric_file),
                Rule::metric => {
                    // We need to avoid the default, else we could use `Metric::try_from_pair`
                    match pair.into_inner().next() {
                        Some(pair) => {
                            let value = Metric::classify_prechecked(pair);
                            $B = $B.metric(value);
                        }
                        _ => (),
                    }
                }
                Rule::engine => match pair.into_inner().next() {
                    Some(pair) => match pair.as_rule() {
                        Rule::nuts => {
                            if let Some(value) = unify_max_depth(pair)? {
                                $max_depth = Some(value);
                            }
                            $state = true;
                        }
                        Rule::r#static => {
                            if let Some(value) = unify_int_time(pair) {
                                $int_time = Some(value);
                            }
                            $state = false;
                        }
                        _ => unreachable!(),
                    },
                    _ => (),
                },
                _ => unreachable!(),
            }
        }
    };
}

fn engine_cond(state: bool, max_depth: Option<i32>, int_time: Option<f64>) -> Engine {
    if state {
        let mut b = NutsBuilder::new();
        if let Some(value) = max_depth {
            b = b.max_depth(value);
        }
        b.build()
    } else {
        let mut b = StaticBuilder::new();
        if let Some(value) = int_time {
            b = b.int_time(value);
        }
        b.build()
    }
}

impl SampleAlgorithm {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::sample_algorithm => {
                let pair = match pair.into_inner().next() {
                    Some(pair) => pair,
                    _ => return Ok(Self::default()),
                };
                match pair.as_rule() {
                    Rule::fixed_param => Ok(Self::FixedParam),
                    Rule::hmc => {
                        // Here, we need to store external states in order to
                        // perform unification on the `Engine` variants.
                        let mut state = true; // true => Nuts, false => Static
                        let mut max_depth: Option<i32> = None;
                        let mut int_time: Option<f64> = None;
                        let mut builder = HmcBuilder::new();
                        unify_hmc_terms!(builder, pair, state, max_depth, int_time);
                        let engine = engine_cond(state, max_depth, int_time);
                        Ok(builder.engine(engine).build())
                    }
                    _ => unreachable!(),
                }
            }

            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }
}

impl FromStr for SampleAlgorithm {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::sample_algorithm_as_type, s) {
            Ok(mut pair) => {
                let pair = pair.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(SampleAlgorithmError(format!("{e:#?}"))),
        }
    }
}

pub(crate) fn try_sample_from_pair(pair: Pair<'_, Rule>) -> Result<Method, ParseGrammarError> {
    match pair.as_rule() {
        Rule::sample => {
            let sample = pair;
            let algorithms = sample
                .clone()
                .into_inner()
                .map(|sample_term| sample_term.into_inner().next().unwrap())
                .filter(|p| match p.as_rule() {
                    Rule::sample_algorithm => true,
                    _ => false,
                });
            let adapts = sample
                .clone()
                .into_inner()
                .map(|sample_term| sample_term.into_inner().next().unwrap())
                .filter(|p| match p.as_rule() {
                    Rule::sample_adapt => true,
                    _ => false,
                });

            // First, we unify the adapts
            let mut builder = SampleAdapt::builder();
            for adapt in adapts {
                unify_sample_adapt_terms!(builder, adapt);
            }
            let adapt = builder.build();

            // Next, we unify the algorithms.
            // Here, we need an extra state to store the algorithm type
            // We also use an `HmcBuilder` to store the state; whether it
            // gets built consumed depends on the algorithm type.
            let mut alg_state = true; // true => Hmc, false => FixedParam
            let mut builder = HmcBuilder::new();
            // Here, we need to store external states in order to
            // perform unification on the `Engine` variants.
            let mut engine_state = true; // true => Nuts, false => Static
            let mut max_depth: Option<i32> = None;
            let mut int_time: Option<f64> = None;
            for algorithm in algorithms {
                match algorithm.into_inner().next() {
                    Some(pair) => match pair.as_rule() {
                        Rule::fixed_param => {
                            alg_state = false;
                        }
                        Rule::hmc => {
                            alg_state = true;
                            unify_hmc_terms!(builder, pair, engine_state, max_depth, int_time);
                        }
                        _ => unreachable!(),
                    },
                    _ => (),
                }
            }
            let algorithm = if !alg_state {
                SampleAlgorithm::FixedParam
            } else {
                let engine = engine_cond(engine_state, max_depth, int_time);
                builder.engine(engine).build()
            };

            // Finally, we unify all the other fields
            let fields = sample
                .into_inner()
                .map(|sample_term| sample_term.into_inner().next().unwrap())
                .filter(|p| match p.as_rule() {
                    Rule::sample_algorithm => false,
                    Rule::sample_adapt => false,
                    _ => true,
                });
            // We use a builder to store states
            let mut builder = SampleBuilder::new();
            for pair in fields {
                match pair.as_rule() {
                    Rule::num_samples => number_arm!(builder, pair, num_samples, i32),
                    Rule::num_warmup => number_arm!(builder, pair, num_warmup, i32),
                    Rule::thin => number_arm!(builder, pair, thin, i32),
                    Rule::num_chains => number_arm!(builder, pair, num_chains, i32),
                    Rule::save_warmup => boolean_arm!(builder, pair, save_warmup),
                    // nothing else is reachable since algorithm and adapt have
                    // been filtered
                    _ => unreachable!(),
                }
            }
            Ok(builder.algorithm(algorithm).adapt(adapt).build())
        }
        r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod metric {
        use super::*;
        use Metric::*;

        #[test]
        fn from_str() {
            assert_eq!("metric".parse::<Metric>().unwrap(), DiagE);
            assert_eq!("metric=unit_e".parse::<Metric>().unwrap(), UnitE);
            assert_eq!("metric=diag_e".parse::<Metric>().unwrap(), DiagE);
            assert_eq!("metric=dense_e".parse::<Metric>().unwrap(), DenseE);
            assert!("".parse::<Metric>().is_err());
        }
    }

    mod engine {
        use super::*;
        use Engine::*;

        #[test]
        fn from_str() {
            assert_eq!("engine".parse::<Engine>().unwrap(), Engine::default());
            assert!("".parse::<Engine>().is_err());

            let lhs = "engine=static int_time int_time=3 int_time=2 int_time"
                .parse::<Engine>()
                .unwrap();
            let rhs = Static { int_time: 2.0_f64 };
            assert_eq!(lhs, rhs);

            assert!("engine=static int_time int_time=3 int_time int_time"
                .parse::<Engine>()
                .is_err());
        }
    }

    mod sample_adapt {
        use super::*;

        #[test]
        fn from_str() {
            let s = "adapt engaged engaged";
            assert!(s.parse::<SampleAdapt>().is_err());
            let s = "adapt engaged engaged=0";
            assert!(s.parse::<SampleAdapt>().is_ok());
            let s = "adapt engaged=0 engaged";
            assert!(s.parse::<SampleAdapt>().is_ok());
            let s = "adapt engaged=1 engaged=0";
            assert!(s.parse::<SampleAdapt>().is_ok());
            let s = "adapt";
            assert!(s.parse::<SampleAdapt>().is_ok());

            let s =
                "adapt engaged engaged=0 engaged engaged=1 engaged gamma engaged gamma delta=0.2 kappa=0.3 t0=4 init_buffer=5 term_buffer=6 window=7 t0=99 t0";

            let adapt = s.parse::<SampleAdapt>();
            assert!(adapt.is_ok());
            assert_eq!(
                adapt.unwrap(),
                SampleAdapt::builder()
                    .delta(0.2)
                    .kappa(0.3)
                    .t0(99.0)
                    .init_buffer(5)
                    .term_buffer(6)
                    .window(7)
                    .build()
            );
        }
    }

    mod sample_algorithm {
        use super::*;

        #[test]
        fn from_str() {
            let s = "algorithm=hmc stepsize=0.5 metric=unit_e engine=nuts max_depth=5 engine=static int_time=3 engine=nuts max_depth=7 engine=static engine=nuts engine=static metric metric=dense_e stepsize_jitter stepsize stepsize_jitter=0.2 stepsize=0.51 engine=nuts";
            let lhs = s.parse::<SampleAlgorithm>().unwrap();
            let rhs = HmcBuilder::new()
                .engine(NutsBuilder::new().max_depth(7))
                .metric(Metric::DenseE)
                .stepsize(0.51)
                .stepsize_jitter(0.2)
                .build();
            assert_eq!(lhs, rhs);

            let s = "algorithm=hmc metric metric_file metric_file=foo.csv";
            let lhs = s.parse::<SampleAlgorithm>().unwrap();
            let rhs = HmcBuilder::new().metric_file("foo.csv").build();
            assert_eq!(lhs, rhs);

            let s = "algorithm=hmc metric=dense_e metric_file metric_file=foo.csv metric_file=bar.txt metric=unit_e";
            let lhs = s.parse::<SampleAlgorithm>().unwrap();
            let rhs = HmcBuilder::new()
                .metric_file("bar.txt")
                .metric(Metric::UnitE)
                .build();
            assert_eq!(lhs, rhs);
        }

        #[test]
        fn metric_file_oddities() {
            let quots = ["'", "\""];
            for quot in quots {
                let files = [
                    format!("{quot}foo.csv{quot}"),
                    format!("{quot}foo    .csv{quot}"),
                    format!("{quot}foo    bar.csv{quot}"),
                    format!("{quot}{quot}"),
                ];
                files.into_iter().for_each(|file| {
                    let s = format!(
                        "algorithm=hmc metric metric_file metric_file={} metric",
                        file
                    );
                    let lhs = s.parse::<SampleAlgorithm>().unwrap();
                    let rhs = HmcBuilder::new().metric_file(file).build();
                    assert_eq!(lhs, rhs);
                });
            }
        }
    }

    mod method {
        use super::*;

        #[test]
        fn from_str() {
            let suffix = "sample algorithm=hmc stepsize=0.5 engine=nuts max_depth=5 engine=static int_time=3 engine=nuts adapt engaged=0 algorithm=fixed_param algorithm=hmc num_samples=5 num_warmup=20 thin num_chains=5 algorithm=hmc metric=unit_e adapt gamma=0.1 delta=0.2 kappa=0.3 algorithm=hmc engine=static thin=2 num_samples=10 algorithm=hmc engine=nuts engine engine=nuts max_depth=10 max_depth=1 max_depth=2 max_depth=3 engine=static engine=nuts metric=dense_e metric stepsize stepsize_jitter=0.1 stepsize_jitter thin adapt algorithm adapt";
            let s = format!("method={}", suffix);
            let lhs = s.parse::<Method>().unwrap();
            let rhs = SampleBuilder::new()
                .num_samples(10)
                .num_warmup(20)
                .thin(2)
                .adapt(
                    SampleAdapt::builder()
                        .engaged(false)
                        .gamma(0.1)
                        .delta(0.2)
                        .kappa(0.3),
                )
                .algorithm(
                    HmcBuilder::new()
                        .engine(NutsBuilder::new().max_depth(3))
                        .metric(Metric::DenseE)
                        .stepsize(0.5)
                        .stepsize_jitter(0.1),
                )
                .num_chains(5)
                .build();
            assert_eq!(lhs, rhs);
            let lhs = suffix.parse::<Method>().unwrap();
            assert_eq!(lhs, rhs);

            let lhs = "method".parse::<Method>().unwrap();
            assert_eq!(lhs, Method::default());

            let lhs = "sample".parse::<Method>().unwrap();
            assert_eq!(lhs, Method::default());

            let lhs = "method=sample".parse::<Method>().unwrap();
            assert_eq!(lhs, Method::default());
        }
    }
}
