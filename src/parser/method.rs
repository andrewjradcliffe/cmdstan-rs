use crate::method::Method;
use crate::parser::diagnose::try_diagnose_from_pair;
use crate::parser::generate_quantities::try_generate_quantities_from_pair;
use crate::parser::laplace::try_laplace_from_pair;
use crate::parser::log_prob::try_log_prob_from_pair;
use crate::parser::optimize::try_optimize_from_pair;
use crate::parser::pathfinder::try_pathfinder_from_pair;
use crate::parser::sample::try_sample_from_pair;
use crate::parser::variational::try_variational_from_pair;
use crate::parser::*;

impl Method {
    pub(crate) fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::method | Rule::method_special_case => match pair.into_inner().next() {
                Some(pair) => match pair.as_rule() {
                    Rule::sample => try_sample_from_pair(pair),
                    Rule::optimize => try_optimize_from_pair(pair),
                    Rule::variational => try_variational_from_pair(pair),
                    Rule::diagnose => try_diagnose_from_pair(pair),
                    Rule::generate_quantities => try_generate_quantities_from_pair(pair),
                    Rule::pathfinder => try_pathfinder_from_pair(pair),
                    Rule::log_prob => try_log_prob_from_pair(pair),
                    Rule::laplace => try_laplace_from_pair(pair),
                    _ => unreachable!(),
                },
                _ => Ok(Self::default()),
            },
            r => Err(RuleError(r)),
        }
    }
}

impl FromStr for Method {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::method_as_type, s) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => error_position!(e, MethodError),
        }
    }
}
