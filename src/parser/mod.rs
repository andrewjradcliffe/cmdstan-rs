use pest::iterators::Pair;
use pest::Parser;
use std::str::FromStr;

#[derive(pest_derive::Parser)]
#[grammar = "parser/base.pest"]
#[grammar = "parser/sample.pest"]
#[grammar = "parser/optimize.pest"]
#[grammar = "parser/variational.pest"]
#[grammar = "parser/diagnose.pest"]
#[grammar = "parser/generate_quantities.pest"]
#[grammar = "parser/pathfinder.pest"]
#[grammar = "parser/logprob.pest"]
#[grammar = "parser/laplace.pest"]
#[grammar = "parser/method.pest"]
#[grammar = "parser/data.pest"]
#[grammar = "parser/random.pest"]
#[grammar = "parser/output.pest"]
#[grammar = "parser/argument_tree.pest"]
pub struct GrammarParser;

#[derive(Debug, PartialEq)]
pub enum ParseGrammarError {
    MetricError(String),
    EngineError(String),
    SampleAdaptError(String),
    SampleAlgorithmError(String),
    OptimizeAlgorithmError(String),
    VariationalAdaptError(String),
    VariationalAlgorithmError(String),
    MethodError(String),
    RuleError(String),
}
use ParseGrammarError::*;

// Common macros
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

mod method;
mod optimize;
mod sample;
mod variational;
