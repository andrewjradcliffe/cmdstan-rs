use crate::method::Method;

/// Options builder for `Method::Pathfinder`.
/// For any option left unspecified, the default value indicated
/// on `Method::Pathfinder` will be supplied.
pub struct PathfinderBuilder {
    init_alpha: Option<f64>,
    tol_obj: Option<f64>,
    tol_rel_obj: Option<f64>,
    tol_grad: Option<f64>,
    tol_rel_grad: Option<f64>,
    tol_param: Option<f64>,
    history_size: Option<i32>,
    num_psis_draws: Option<i32>,
    num_paths: Option<i32>,
    save_single_paths: Option<bool>,
    max_lbfgs_iters: Option<i32>,
    num_draws: Option<i32>,
    num_elbo_draws: Option<i32>,
}

impl PathfinderBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            init_alpha: None,
            tol_obj: None,
            tol_rel_obj: None,
            tol_grad: None,
            tol_rel_grad: None,
            tol_param: None,
            history_size: None,
            num_psis_draws: None,
            num_paths: None,
            save_single_paths: None,
            max_lbfgs_iters: None,
            num_draws: None,
            num_elbo_draws: None,
        }
    }
    insert_field!(init_alpha, f64);
    insert_field!(tol_obj, f64);
    insert_field!(tol_rel_obj, f64);
    insert_field!(tol_grad, f64);
    insert_field!(tol_rel_grad, f64);
    insert_field!(tol_param, f64);
    insert_field!(history_size, i32);
    insert_field!(num_psis_draws, i32);
    insert_field!(num_paths, i32);
    insert_field!(save_single_paths, bool);
    insert_field!(max_lbfgs_iters, i32);
    insert_field!(num_draws, i32);
    insert_field!(num_elbo_draws, i32);
    pub fn build(self) -> Method {
        let init_alpha = self.init_alpha.unwrap_or(0.001);
        let tol_obj = self.tol_obj.unwrap_or(9.9999999999999998e-13);
        let tol_rel_obj = self.tol_rel_obj.unwrap_or(10_000.0);
        let tol_grad = self.tol_grad.unwrap_or(1e-8);
        let tol_rel_grad = self.tol_rel_grad.unwrap_or(10_000_000.0);
        let tol_param = self.tol_param.unwrap_or(1e-8);
        let history_size = self.history_size.unwrap_or(5);
        let num_psis_draws = self.num_psis_draws.unwrap_or(1000);
        let num_paths = self.num_paths.unwrap_or(4);
        let save_single_paths = self.save_single_paths.unwrap_or(false);
        let max_lbfgs_iters = self.max_lbfgs_iters.unwrap_or(1000);
        let num_draws = self.num_draws.unwrap_or(1000);
        let num_elbo_draws = self.num_elbo_draws.unwrap_or(25);
        Method::Pathfinder {
            init_alpha,
            tol_obj,
            tol_rel_obj,
            tol_grad,
            tol_rel_grad,
            tol_param,
            history_size,
            num_psis_draws,
            num_paths,
            save_single_paths,
            max_lbfgs_iters,
            num_draws,
            num_elbo_draws,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = PathfinderBuilder::new()
            .init_alpha(0.1)
            .tol_obj(0.2)
            .tol_rel_obj(0.3)
            .tol_grad(0.4)
            .tol_rel_grad(0.5)
            .tol_param(0.6)
            .history_size(1)
            .num_psis_draws(2)
            .num_paths(3)
            .save_single_paths(true)
            .max_lbfgs_iters(4)
            .num_draws(5)
            .num_elbo_draws(6)
            .build();
        assert_eq!(
            x,
            Method::Pathfinder {
                init_alpha: 0.1,
                tol_obj: 0.2,
                tol_rel_obj: 0.3,
                tol_grad: 0.4,
                tol_rel_grad: 0.5,
                tol_param: 0.6,
                history_size: 1,
                num_psis_draws: 2,
                num_paths: 3,
                save_single_paths: true,
                max_lbfgs_iters: 4,
                num_draws: 5,
                num_elbo_draws: 6,
            }
        );

        let x = PathfinderBuilder::new().build();
        assert_eq!(
            x,
            Method::Pathfinder {
                init_alpha: 0.001,
                tol_obj: 9.9999999999999998e-13,
                tol_rel_obj: 10_000.0,
                tol_grad: 1e-8,
                tol_rel_grad: 10_000_000.0,
                tol_param: 1e-8,
                history_size: 5,
                num_psis_draws: 1000,
                num_paths: 4,
                save_single_paths: false,
                max_lbfgs_iters: 1000,
                num_draws: 1000,
                num_elbo_draws: 25,
            }
        );
    }
}
