use crate::method::{Method, PathfinderBuilder};
use crate::parser::*;

pub(crate) fn try_pathfinder_from_pair(pair: Pair<'_, Rule>) -> Result<Method, ParseGrammarError> {
    match pair.as_rule() {
        Rule::pathfinder => {
            let pairs = pair.into_inner();
            // We use the builder to hold state during unification.
            let mut builder = PathfinderBuilder::new();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::init_alpha => number_arm!(builder, pair, init_alpha, f64),
                    Rule::tol_obj => number_arm!(builder, pair, tol_obj, f64),
                    Rule::tol_rel_obj => number_arm!(builder, pair, tol_rel_obj, f64),
                    Rule::tol_grad => number_arm!(builder, pair, tol_grad, f64),
                    Rule::tol_rel_grad => number_arm!(builder, pair, tol_rel_grad, f64),
                    Rule::tol_param => number_arm!(builder, pair, tol_param, f64),
                    Rule::history_size => number_arm!(builder, pair, history_size, i32),
                    Rule::num_psis_draws => number_arm!(builder, pair, num_psis_draws, i32),
                    Rule::num_paths => number_arm!(builder, pair, num_paths, i32),
                    Rule::save_single_paths => boolean_arm!(builder, pair, save_single_paths),
                    Rule::max_lbfgs_iters => number_arm!(builder, pair, max_lbfgs_iters, i32),
                    Rule::num_draws => number_arm!(builder, pair, num_draws, i32),
                    Rule::num_elbo_draws => number_arm!(builder, pair, num_elbo_draws, i32),
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
            let rhs = PathfinderBuilder::new().build();
            assert_eq!("pathfinder".parse::<Method>().unwrap(), rhs);
            assert_eq!("method=pathfinder".parse::<Method>().unwrap(), rhs);

            let s = "method=pathfinder init_alpha=1 tol_obj=2 tol_grad=3 tol_rel_grad tol_rel_grad=4 history_size=5 history_size=6 history_size num_draws num_draws=10 num_draws=11 num_elbo_draws=50 num_elbo_draws=42 num_paths=999 save_single_paths=0 save_single_paths=1 num_psis_draws=5";
            let rhs = PathfinderBuilder::new()
                .init_alpha(1.0)
                .tol_obj(2.0)
                .tol_grad(3.0)
                .tol_rel_grad(4.0)
                .history_size(6)
                .num_draws(11)
                .num_elbo_draws(42)
                .num_paths(999)
                .save_single_paths(true)
                .num_psis_draws(5)
                .build();
            assert_eq!(s.parse::<Method>().unwrap(), rhs);

            assert!("method=pathfinder init_alpha init_alpha"
                .parse::<Method>()
                .is_err());
        }
    }
}
