use crate::log_prob::*;
use crate::method::Method;
use crate::parser::*;

pub(crate) fn try_log_prob_from_pair(pair: Pair<'_, Rule>) -> Result<Method, ParseGrammarError> {
    match pair.as_rule() {
        Rule::log_prob => {
            let pairs = pair.into_inner();
            // We use the builder to hold state during unification.
            let mut builder = LogProbBuilder::new();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::jacobian => boolean_arm!(builder, pair, jacobian),
                    // This will cause allocations for each path string,
                    // but it repetitions of path will be very rare.
                    Rule::unconstrained_params => path_arm!(builder, pair, unconstrained_params),
                    Rule::constrained_params => path_arm!(builder, pair, constrained_params),
                    _ => unreachable!(),
                }
            }
            Ok(builder.build())
        }
        r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod method {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = LogProbBuilder::new().build();
            assert_eq!("log_prob".parse::<Method>().unwrap(), rhs);
            assert_eq!("method=log_prob".parse::<Method>().unwrap(), rhs);

            let s = "method=log_prob jacobian jacobian=0 jacobian=1 unconstrained_params=foo.bar unconstrained_params unconstrained_params=bar.baz constrained_params=foo.bar constrained_params jacobian=0";
            let rhs = LogProbBuilder::new()
                .jacobian(false)
                .unconstrained_params("bar.baz")
                .constrained_params("foo.bar")
                .build();
            assert_eq!(s.parse::<Method>().unwrap(), rhs);
        }
    }
}
