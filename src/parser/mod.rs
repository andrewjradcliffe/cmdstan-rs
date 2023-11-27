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

#[derive(Debug)]
pub enum ParseGrammarError {
    MetricError(String),
    EngineError(String),
    SampleAdaptError(String),
    SampleAlgorithmError(String),
    MethodError(String),
    RuleError(String),
}
use ParseGrammarError::*;

mod method;
mod sample;
