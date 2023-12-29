use crate::method::{LaplaceBuilder, Method};
use crate::parser::*;

pub(crate) fn try_laplace_from_pair(pair: Pair<'_, Rule>) -> Result<Method, ParseGrammarError> {
    match pair.as_rule() {
        Rule::laplace => {
            let pairs = pair.into_inner();
            // We use the builder to hold state during unification.
            let mut builder = LaplaceBuilder::new();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::jacobian => boolean_arm!(builder, pair, jacobian),
                    // This will cause allocations for each path string,
                    // but it repetitions of path will be very rare.
                    Rule::mode => path_arm!(builder, pair, mode),
                    Rule::draws => number_arm!(builder, pair, draws, i32),
                    _ => unreachable!(),
                }
            }
            Ok(builder.build())
        }
        r => Err(RuleError(r)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod method {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = LaplaceBuilder::new().build();
            assert_eq!("laplace".parse::<Method>().unwrap(), rhs);
            assert_eq!("method=laplace".parse::<Method>().unwrap(), rhs);

            let s = "method=laplace jacobian jacobian=0 jacobian=1 mode=foo.bar mode mode=bar.baz draws=42 draws jacobian=0";
            let rhs = LaplaceBuilder::new()
                .jacobian(false)
                .mode("bar.baz")
                .draws(42)
                .build();
            assert_eq!(s.parse::<Method>().unwrap(), rhs);
        }
    }
}
