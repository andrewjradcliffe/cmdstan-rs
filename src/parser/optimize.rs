use crate::method::{Method, OptimizeBuilder};
use crate::optimize::*;
use crate::parser::*;

impl_from_str! { OptimizeAlgorithm, OptimizeAlgorithmError, optimize_algorithm_as_type }

macro_rules! unify_bfgs_terms {
    ($B:ident, $bfgs:ident) => {
        let pairs = $bfgs.into_inner();
        for pair in pairs {
            match pair.as_rule() {
                Rule::init_alpha => number_arm!($B, pair, init_alpha, f64),
                Rule::tol_obj => number_arm!($B, pair, tol_obj, f64),
                Rule::tol_rel_obj => number_arm!($B, pair, tol_rel_obj, f64),
                Rule::tol_grad => number_arm!($B, pair, tol_grad, f64),
                Rule::tol_rel_grad => number_arm!($B, pair, tol_rel_grad, f64),
                Rule::tol_param => number_arm!($B, pair, tol_param, f64),
                _ => unreachable!(),
            }
        }
    };
}
macro_rules! unify_lbfgs_terms {
    ($B:ident, $lbfgs:ident) => {
        let pairs = $lbfgs.into_inner();
        for pair in pairs {
            match pair.as_rule() {
                Rule::init_alpha => number_arm!($B, pair, init_alpha, f64),
                Rule::tol_obj => number_arm!($B, pair, tol_obj, f64),
                Rule::tol_rel_obj => number_arm!($B, pair, tol_rel_obj, f64),
                Rule::tol_grad => number_arm!($B, pair, tol_grad, f64),
                Rule::tol_rel_grad => number_arm!($B, pair, tol_rel_grad, f64),
                Rule::tol_param => number_arm!($B, pair, tol_param, f64),
                Rule::history_size => number_arm!($B, pair, history_size, i32),
                _ => unreachable!(),
            }
        }
    };
}

impl OptimizeAlgorithm {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::optimize_algorithm => {
                let pair = match pair.into_inner().next() {
                    Some(pair) => pair,
                    _ => return Ok(Self::default()),
                };
                match pair.as_rule() {
                    Rule::newton => Ok(Self::Newton),
                    Rule::bfgs => {
                        let mut builder = BfgsBuilder::new();
                        unify_bfgs_terms!(builder, pair);
                        Ok(builder.build())
                    }
                    Rule::lbfgs => {
                        let mut builder = LbfgsBuilder::new();
                        unify_lbfgs_terms!(builder, pair);
                        Ok(builder.build())
                    }
                    _ => unreachable!(),
                }
            }
            r => Err(RuleError(r)),
        }
    }
}

pub(crate) fn try_optimize_from_pair(pair: Pair<'_, Rule>) -> Result<Method, ParseGrammarError> {
    match pair.as_rule() {
        Rule::optimize => {
            // We need 3 states to handle the 3 variants.
            // 0 => Bfgs, 1 => Lbfgs, 2 => Newton
            let mut alg_state: u8 = 1;
            // We use builders to hold state for the respective algorithm variants
            let mut bfgs_builder = BfgsBuilder::new();
            let mut lbfgs_builder = LbfgsBuilder::new();
            // We use another builder to hold state for the method variant
            let mut opt_builder = OptimizeBuilder::new();
            let pairs = pair.into_inner();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::optimize_algorithm => {
                        if let Some(pair) = pair.into_inner().next() {
                            match pair.as_rule() {
                                Rule::bfgs => {
                                    alg_state = 0;
                                    unify_bfgs_terms!(bfgs_builder, pair);
                                }
                                Rule::lbfgs => {
                                    alg_state = 1;
                                    unify_lbfgs_terms!(lbfgs_builder, pair);
                                }
                                Rule::newton => {
                                    alg_state = 2;
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    Rule::jacobian => boolean_arm!(opt_builder, pair, jacobian),
                    Rule::iter => number_arm!(opt_builder, pair, iter, i32),
                    Rule::save_iterations => boolean_arm!(opt_builder, pair, save_iterations),
                    _ => unreachable!(),
                }
            }

            let algorithm = match alg_state {
                0 => bfgs_builder.build(),
                1 => lbfgs_builder.build(),
                2 => OptimizeAlgorithm::Newton,
                _ => unreachable!(),
            };

            Ok(opt_builder.algorithm(algorithm).build())
        }
        r => Err(RuleError(r)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod optimize_algorithm {
        use super::*;

        #[test]
        fn from_str() {
            let s = "algorithm=lbfgs history_size=10 init_alpha init_alpha=0.01 init_alpha=0.02 tol_obj=5 tol_obj tol_rel_obj=10 tol_obj=10 tol_param=20 history_size=100 tol_rel_grad=30 tol_grad=40";
            let lhs = s.parse::<OptimizeAlgorithm>().unwrap();
            let rhs = LbfgsBuilder::new()
                .init_alpha(0.02)
                .tol_obj(10.0)
                .tol_rel_obj(10.0)
                .tol_param(20.0)
                .tol_rel_grad(30.0)
                .tol_grad(40.0)
                .history_size(100)
                .build();
            assert_eq!(lhs, rhs);

            let s = "algorithm";
            assert_eq!(
                s.parse::<OptimizeAlgorithm>().unwrap(),
                LbfgsBuilder::new().build()
            );

            let s = "algorithm=newton";
            assert_eq!(
                s.parse::<OptimizeAlgorithm>().unwrap(),
                OptimizeAlgorithm::Newton
            );

            let s = "algorithm=bfgs init_alpha init_alpha=0.01 init_alpha=0.02 tol_obj=5 tol_obj tol_rel_obj=10 tol_obj=10 tol_param=20 tol_rel_grad=30 tol_grad=40";
            let lhs = s.parse::<OptimizeAlgorithm>().unwrap();
            let rhs = BfgsBuilder::new()
                .init_alpha(0.02)
                .tol_obj(10.0)
                .tol_rel_obj(10.0)
                .tol_param(20.0)
                .tol_rel_grad(30.0)
                .tol_grad(40.0)
                .build();
            assert_eq!(lhs, rhs);
        }
    }

    mod method {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = OptimizeBuilder::new().build();
            assert_eq!("optimize".parse::<Method>().unwrap(), rhs);
            assert_eq!("method=optimize".parse::<Method>().unwrap(), rhs);

            assert!("method=optimize optimize".parse::<Method>().is_err());
            assert!("method= optimize".parse::<Method>().is_err());

            let rhs = OptimizeBuilder::new()
                .algorithm(BfgsBuilder::new().init_alpha(0.1))
                .build();
            let s = "method=optimize algorithm=bfgs init_alpha=0.1 algorithm=lbfgs algorithm=newton algorithm=bfgs";
            assert_eq!(s.parse::<Method>().unwrap(), rhs);
            let s = "method=optimize algorithm=bfgs init_alpha=0.1 algorithm=lbfgs init_alpha=0.2 algorithm=newton algorithm=bfgs";
            assert_eq!(s.parse::<Method>().unwrap(), rhs);

            let rhs = OptimizeBuilder::new()
                .algorithm(BfgsBuilder::new())
                .jacobian(true)
                .iter(10)
                .save_iterations(true)
                .build();
            let s = "method=optimize algorithm=lbfgs init_alpha=0.2 iter algorithm=bfgs algorithm iter=10 save_iterations=+1 jacobian=-0 jacobian=+0 jacobian=+1 jacobian save_iterations jacobian save_iterations";
            assert_eq!(s.parse::<Method>().unwrap(), rhs);
        }
    }
}
