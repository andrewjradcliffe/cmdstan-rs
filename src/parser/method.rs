use crate::method::Method;
use crate::parser::optimize::try_optimize_from_pair;
use crate::parser::sample::try_sample_from_pair;
use crate::parser::variational::try_variational_from_pair;
use crate::parser::*;

impl Method {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::method => match pair.into_inner().next() {
                Some(pair) => {
                    let pair = pair.into_inner().next().unwrap();
                    match pair.as_rule() {
                        Rule::sample => try_sample_from_pair(pair),
                        Rule::optimize => try_optimize_from_pair(pair),
                        Rule::variational => try_variational_from_pair(pair),
                        _ => todo!(),
                    }
                }
                _ => Ok(Self::default()),
            },
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
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
            Err(e) => Err(MethodError(format!("{e:#?}"))),
        }
    }
}
