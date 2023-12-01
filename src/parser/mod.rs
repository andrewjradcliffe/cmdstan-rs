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
    MetricError(String),
    EngineError(String),
    SampleAdaptError(String),
    SampleAlgorithmError(String),
    OptimizeAlgorithmError(String),
    VariationalAdaptError(String),
    VariationalAlgorithmError(String),
    DiagnoseTestError(String),
    MethodError(String),
    OutputError(String),
    RandomError(String),
    DataError(String),
    ArgumentTreeError(String),
    RuleError(String),
}
use ParseGrammarError::*;

// Common macros
macro_rules! number_arm {
    ($B:ident, $P:ident, $F:ident, $T:ty) => {
        if let Some(pair) = $P.into_inner().next() {
            let value = pair.as_str().parse::<$T>().unwrap();
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
