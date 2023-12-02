use crate::diagnose::*;
use crate::method::Method;
use crate::parser::*;

impl_from_str! { DiagnoseTest, DiagnoseTestError, diagnose_test_as_type }

fn unify_gradient_fields(pair: Pair<'_, Rule>) -> (Option<f64>, Option<f64>) {
    let pairs = pair.into_inner();
    let mut epsilon: Option<f64> = None;
    let mut error: Option<f64> = None;
    for pair in pairs {
        match pair.as_rule() {
            Rule::epsilon => {
                if let Some(pair) = pair.into_inner().next() {
                    let value = pair.as_str().parse::<f64>().unwrap();
                    epsilon = Some(value);
                }
            }
            Rule::error => {
                if let Some(pair) = pair.into_inner().next() {
                    let value = pair.as_str().parse::<f64>().unwrap();
                    error = Some(value);
                }
            }
            _ => unreachable!(),
        }
    }
    (epsilon, error)
}

macro_rules! unify_gradient_terms {
    ($B:ident, $P:ident) => {
        let (epsilon, error) = unify_gradient_fields($P);
        if let Some(epsilon) = epsilon {
            $B = $B.epsilon(epsilon);
        }
        if let Some(error) = error {
            $B = $B.error(error);
        }
    };
}

impl DiagnoseTest {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::diagnose_test => {
                let variant = match pair.into_inner().next() {
                    Some(pair) => match pair.as_rule() {
                        Rule::gradient => {
                            let mut builder = GradientBuilder::new();
                            unify_gradient_terms!(builder, pair);
                            builder.build()
                        }
                        _ => unreachable!(),
                    },
                    _ => Self::default(),
                };
                Ok(variant)
            }
            r => Err(RuleError(r)),
        }
    }
}

pub(crate) fn try_diagnose_from_pair(pair: Pair<'_, Rule>) -> Result<Method, ParseGrammarError> {
    match pair.as_rule() {
        Rule::diagnose => {
            let pairs = pair.into_inner();

            // We set default states prior to unification.
            // Only a single variant exists on the sum type, thus, we simplify.
            let mut builder = GradientBuilder::new();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::diagnose_test => {
                        if let Some(pair) = pair.into_inner().next() {
                            match pair.as_rule() {
                                Rule::gradient => {
                                    unify_gradient_terms!(builder, pair);
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Ok(DiagnoseBuilder::new().test(builder).build())
        }
        r => Err(RuleError(r)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod diagnose_test {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = DiagnoseTest::default();
            let s = "test";
            assert_eq!(s.parse::<DiagnoseTest>().unwrap(), rhs);
            let s = "test=gradient";
            assert_eq!(s.parse::<DiagnoseTest>().unwrap(), rhs);
            let s = "test=gradient epsilon error";
            assert_eq!(s.parse::<DiagnoseTest>().unwrap(), rhs);

            let s = "test=gradient epsilon=0.1 error=0.2 epsilon=0.2";
            assert_eq!(
                s.parse::<DiagnoseTest>().unwrap(),
                GradientBuilder::new().epsilon(0.2).error(0.2).build()
            );
        }
    }

    mod method {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = DiagnoseBuilder::new().build();
            assert_eq!("diagnose".parse::<Method>().unwrap(), rhs);
            assert_eq!("method=diagnose".parse::<Method>().unwrap(), rhs);
            assert_eq!("method=diagnose test".parse::<Method>().unwrap(), rhs);
            assert_eq!(
                "method=diagnose test=gradient".parse::<Method>().unwrap(),
                rhs
            );

            let rhs = DiagnoseBuilder::new()
                .test(GradientBuilder::new().epsilon(0.1).error(0.5))
                .build();

            let s = "method=diagnose test=gradient epsilon=0.2 epsilon=0.3 test=gradient epsilon=0.5 error=0.4 test=gradient epsilon=0.1 test test=gradient error=0.6 test=gradient error=0.5";
            assert_eq!(s.parse::<Method>().unwrap(), rhs);

            let s = "method=diagnose test test";
            assert!(s.parse::<Method>().is_err());
        }
    }
}
