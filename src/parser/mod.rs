use pest::{error::InputLocation, iterators::Pair, Parser};
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

#[derive(pest_derive::Parser)]
#[grammar = "parser/base.pest"]
#[grammar = "parser/sample.pest"]
#[grammar = "parser/optimize.pest"]
#[grammar = "parser/variational.pest"]
#[grammar = "parser/diagnose.pest"]
#[grammar = "parser/generate_quantities.pest"]
#[grammar = "parser/pathfinder.pest"]
#[grammar = "parser/log_prob.pest"]
#[grammar = "parser/laplace.pest"]
#[grammar = "parser/method.pest"]
#[grammar = "parser/data.pest"]
#[grammar = "parser/random.pest"]
#[grammar = "parser/output.pest"]
#[grammar = "parser/argument_tree.pest"]
pub struct GrammarParser;

#[derive(Debug, PartialEq)]
pub enum ParseGrammarError {
    IntError(ParseIntError),
    FloatError(ParseFloatError),
    MetricError(usize),
    EngineError(usize),
    SampleAdaptError(usize),
    SampleAlgorithmError(usize),
    OptimizeAlgorithmError(usize),
    VariationalAdaptError(usize),
    VariationalAlgorithmError(usize),
    DiagnoseTestError(usize),
    MethodError(usize),
    OutputError(usize),
    RandomError(usize),
    DataError(usize),
    ArgumentTreeError(usize),
    TopLevelDuplicate(&'static str),
    MethodNotSpecified,
    RuleError(Rule),
}
use ParseGrammarError::*;

impl From<ParseIntError> for ParseGrammarError {
    fn from(e: ParseIntError) -> Self {
        IntError(e)
    }
}
impl From<ParseFloatError> for ParseGrammarError {
    fn from(e: ParseFloatError) -> Self {
        FloatError(e)
    }
}

impl fmt::Display for ParseGrammarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (word, pos) = match self {
            MetricError(n) => ("method", n),
            EngineError(n) => ("engine", n),
            SampleAdaptError(n) => ("(sample) adapt", n),
            SampleAlgorithmError(n) => ("(sample) algorithm", n),
            OptimizeAlgorithmError(n) => ("(optimize) algorithm", n),
            VariationalAdaptError(n) => ("(variational) adapt", n),
            VariationalAlgorithmError(n) => ("(variational) algorithm", n),
            DiagnoseTestError(n) => ("test", n),
            MethodError(n) => ("method", n),
            OutputError(n) => ("output", n),
            RandomError(n) => ("random", n),
            DataError(n) => ("data", n),
            ArgumentTreeError(n) => ("top-level", n),
            RuleError(r) => {
                return write!(f, "internal parsing error: {:?}", r);
            }
            TopLevelDuplicate(s) => {
                return write!(f, "{} was declared more than once", s);
            }
            MethodNotSpecified => {
                return write!(f, "A method must be specified!");
            }
            IntError(e) => {
                return write!(f, "{}", e);
            }
            FloatError(e) => {
                return write!(f, "{}", e);
            }
        };
        write!(
            f,
            "{} does not conform to grammar at position {}",
            word, pos
        )
    }
}
impl std::error::Error for ParseGrammarError {}

// Common macros
macro_rules! number_arm {
    ($B:ident, $P:ident, $F:ident, $T:ty) => {
        if let Some(pair) = $P.into_inner().next() {
            let value = pair.as_str().parse::<$T>()?;
            $B = $B.$F(value);
        }
    };
}
macro_rules! boolean_arm {
    ($B:ident, $P:ident, $F:ident) => {
        if let Some(pair) = $P.into_inner().next() {
            let value = match pair.as_rule() {
                Rule::r#true => true,
                Rule::r#false => false,
                _ => unreachable!(),
            };
            $B = $B.$F(value);
        }
    };
}
macro_rules! path_arm {
    ($B:ident, $P:ident, $F:ident) => {
        if let Some(pair) = $P.into_inner().next() {
            $B = $B.$F(pair.as_str());
        }
    };
}

macro_rules! error_position {
    ($e:ident, $E:ident) => {
        match $e.location {
            InputLocation::Pos(r) => Err($E(r)),
            InputLocation::Span((_, r)) => Err($E(r)),
        }
    };
}

macro_rules! impl_from_str {
    { $T:ident, $E:ident, $R:ident } => {
        impl FromStr for $T {
            type Err = ParseGrammarError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match GrammarParser::parse(Rule::$R, s) {
                    Ok(mut pair) => {
                        let pair = pair.next().unwrap().into_inner().next().unwrap();
                        Self::try_from_pair(pair)
                    }
                    Err(e) => error_position!(e, $E),
                }
            }
        }
    }
}

mod argument_tree;
mod diagnose;
mod generate_quantities;
mod laplace;
mod log_prob;
mod method;
mod optimize;
mod pathfinder;
mod sample;
mod variational;
