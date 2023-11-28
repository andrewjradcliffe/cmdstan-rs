use crate::method::Method;
use crate::parser::*;
use crate::variational::*;

impl FromStr for VariationalAdapt {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::variational_adapt_as_type, s) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(VariationalAdaptError(format!("{e:#?}"))),
        }
    }
}

macro_rules! unify_variational_adapt_terms {
    ($B:ident, $P:ident) => {
        let pairs = $P.into_inner().map(|p| p.into_inner().next().unwrap());
        for pair in pairs {
            match pair.as_rule() {
                Rule::engaged => boolean_arm!($B, pair, engaged),
                Rule::iter => number_arm!($B, pair, iter, i32),
                _ => unreachable!(),
            }
        }
    };
}

impl VariationalAdapt {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::variational_adapt => {
                let mut builder = Self::builder();
                unify_variational_adapt_terms!(builder, pair);
                Ok(builder.build())
            }
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }
}

impl FromStr for VariationalAlgorithm {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::variational_algorithm_as_type, s) {
            Ok(mut pair) => {
                let pair = pair.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(VariationalAlgorithmError(format!("{e:#?}"))),
        }
    }
}

impl VariationalAlgorithm {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::variational_algorithm => {
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
    // `Rule::variational_algorithm`.
    #[inline]
    fn classify_prechecked(pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::meanfield => Self::MeanField,
            Rule::fullrank => Self::FullRank,
            _ => unreachable!(),
        }
    }
}

pub(crate) fn try_variational_from_pair(pair: Pair<'_, Rule>) -> Result<Method, ParseGrammarError> {
    match pair.as_rule() {
        Rule::variational => {
            let pairs = pair
                .into_inner()
                .map(|variational_term| variational_term.into_inner().next().unwrap());
            // We set default states prior to unification
            let mut adapt_builder = VariationalAdapt::builder();
            let mut alg_state = VariationalAlgorithm::default();
            let mut var_builder = VariationalBuilder::new();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::variational_algorithm => match pair.into_inner().next() {
                        Some(pair) => {
                            alg_state = VariationalAlgorithm::classify_prechecked(pair);
                        }
                        _ => (),
                    },
                    Rule::variational_adapt => {
                        unify_variational_adapt_terms!(adapt_builder, pair);
                    }
                    Rule::iter => number_arm!(var_builder, pair, iter, i32),
                    Rule::grad_samples => number_arm!(var_builder, pair, grad_samples, i32),
                    Rule::elbo_samples => number_arm!(var_builder, pair, elbo_samples, i32),
                    Rule::eta => number_arm!(var_builder, pair, eta, f64),
                    Rule::tol_rel_obj => number_arm!(var_builder, pair, tol_rel_obj, f64),
                    Rule::eval_elbo => number_arm!(var_builder, pair, eval_elbo, i32),
                    Rule::output_samples => number_arm!(var_builder, pair, output_samples, i32),
                    _ => unreachable!(),
                }
            }
            Ok(var_builder
                .algorithm(alg_state)
                .adapt(adapt_builder)
                .build())
        }
        r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod variational_algorithm {
        use super::*;

        #[test]
        fn from_str() {
            let s = "algorithm";
            let lhs = s.parse::<VariationalAlgorithm>().unwrap();
            assert_eq!(lhs, VariationalAlgorithm::default());

            let s = "algorithm=meanfield";
            let lhs = s.parse::<VariationalAlgorithm>().unwrap();
            assert_eq!(lhs, VariationalAlgorithm::MeanField);

            let s = "algorithm=fullrank";
            let lhs = s.parse::<VariationalAlgorithm>().unwrap();
            assert_eq!(lhs, VariationalAlgorithm::FullRank);
        }
    }

    mod variational_adapt {
        use super::*;

        #[test]
        fn from_str() {
            let s = "adapt";
            let lhs = s.parse::<VariationalAdapt>().unwrap();
            assert_eq!(lhs, VariationalAdapt::default());

            let s = "adapt engaged=0 iter=25 engaged iter=17";

            let lhs = s.parse::<VariationalAdapt>().unwrap();
            let rhs = VariationalAdapt::builder().engaged(false).iter(17).build();
            assert_eq!(lhs, rhs);
        }
    }

    mod method {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = VariationalBuilder::new().build();
            assert_eq!("variational".parse::<Method>().unwrap(), rhs);
            assert_eq!("method=variational".parse::<Method>().unwrap(), rhs);

            assert!("method=variational variational".parse::<Method>().is_err());
            assert!("method= variational".parse::<Method>().is_err());

            let rhs = VariationalBuilder::new()
                .algorithm(VariationalAlgorithm::FullRank)
                .adapt(VariationalAdapt::builder().engaged(false).iter(17))
                .eta(0.5)
                .iter(42)
                .eval_elbo(10000)
                .grad_samples(123)
                .output_samples(456)
                .build();
            let s = "method=variational algorithm=fullrank adapt engaged=0 eta=0.5 adapt engaged=1 iter=25 eval_elbo=10000 iter=42 adapt engaged=0 iter=17 grad_samples=123 output_samples=456";
            assert_eq!(s.parse::<Method>().unwrap(), rhs);

            // This tests an ambiguity in the grammar definition. The iter without
            // associated value is a declaration within the preceding adapt block.
            // This causes the subsequent adapt declaration to be an error, as
            // there is no valid separator (i.e. any other declaration than an adapt block).
            //
            // This ambiguity is present due to the use of iter as a field name in both
            // the parent sum type variant and the child product type.
            let s = "method=variational adapt engaged=0 iter=17 iter adapt iter=42";
            assert!(s.parse::<Method>().is_err());
            // Here is an example with a valid separator.
            let s = "method=variational adapt engaged=0 iter=17 eta adapt iter=42";
            assert!(s.parse::<Method>().is_ok());
        }
    }
}
