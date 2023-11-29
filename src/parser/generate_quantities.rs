use crate::generate_quantities::*;
use crate::method::Method;
use crate::parser::*;

pub(crate) fn try_generate_quantities_from_pair(
    pair: Pair<'_, Rule>,
) -> Result<Method, ParseGrammarError> {
    match pair.as_rule() {
        Rule::generate_quantities => {
            let builder = pair
                .into_inner()
                .last()
                .map(|pair| GenerateQuantitiesBuilder::new().fitted_params(pair.as_str()))
                .unwrap_or_else(|| GenerateQuantitiesBuilder::new());
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
            let rhs = GenerateQuantitiesBuilder::new().build();
            assert_eq!("generate_quantities".parse::<Method>().unwrap(), rhs);
            assert_eq!("method=generate_quantities".parse::<Method>().unwrap(), rhs);
            assert_eq!(
                "method=generate_quantities fitted_params"
                    .parse::<Method>()
                    .unwrap(),
                rhs
            );

            let rhs = GenerateQuantitiesBuilder::new()
                .fitted_params("foo.bar.baz")
                .build();
            let s = "method=generate_quantities fitted_params fitted_params=foo.bar fitted_params=foo.bar.baz";
            assert_eq!(s.parse::<Method>().unwrap(), rhs);
        }
    }
}
