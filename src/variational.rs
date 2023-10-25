use crate::method::Method;
use std::fmt::Write;

/// Options builder for `Method::Variational`.
/// For any option left unspecified, the default value indicated
/// on `Method::Variational` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct VariationalBuilder {
    algorithm: Option<VariationalAlgorithm>,
    iter: Option<i32>,
    grad_samples: Option<i32>,
    elbo_samples: Option<i32>,
    eta: Option<f64>,
    adapt: Option<VariationalAdapt>,
    tol_rel_obj: Option<f64>,
    eval_elbo: Option<i32>,
    output_samples: Option<i32>,
}
impl VariationalBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            algorithm: None,
            iter: None,
            grad_samples: None,
            elbo_samples: None,
            eta: None,
            adapt: None,
            tol_rel_obj: None,
            eval_elbo: None,
            output_samples: None,
        }
    }
    insert_field!(algorithm, VariationalAlgorithm);
    insert_field!(iter, i32);
    insert_field!(grad_samples, i32);
    insert_field!(elbo_samples, i32);
    insert_field!(eta, f64);
    insert_field!(adapt, VariationalAdapt);
    insert_field!(tol_rel_obj, f64);
    insert_field!(eval_elbo, i32);
    insert_field!(output_samples, i32);
    /// Build the `Method::Variational` instance.
    pub fn build(self) -> Method {
        let algorithm = self.algorithm.unwrap_or_default();
        let iter = self.iter.unwrap_or(10000);
        let grad_samples = self.grad_samples.unwrap_or(1);
        let elbo_samples = self.elbo_samples.unwrap_or(100);
        let eta = self.eta.unwrap_or(1.0);
        let adapt = self.adapt.unwrap_or_default();
        let tol_rel_obj = self.tol_rel_obj.unwrap_or(0.01);
        let eval_elbo = self.eval_elbo.unwrap_or(100);
        let output_samples = self.output_samples.unwrap_or(1000);
        Method::Variational {
            algorithm,
            iter,
            grad_samples,
            elbo_samples,
            eta,
            adapt,
            tol_rel_obj,
            eval_elbo,
            output_samples,
        }
    }
}

/// Variational inference algorithm. Defaults to `MeanField`.
#[derive(Debug, Default, PartialEq, Clone)]
pub enum VariationalAlgorithm {
    /// mean-field approximation
    #[default]
    MeanField,
    /// full-rank covariance
    FullRank,
}
impl VariationalAlgorithm {
    pub fn command_fragment(&self) -> String {
        match &self {
            Self::MeanField => "algorithm=meanfield",
            Self::FullRank => "algorithm=fullrank",
        }
        .to_string()
    }
}

/// Eta Adaptation for Variational Inference
#[derive(Debug, PartialEq, Clone)]
pub struct VariationalAdapt {
    /// Boolean flag for eta adaptation.
    /// Valid values: [0, 1]
    /// Defaults to 1
    pub engaged: bool,
    /// Number of iterations for eta adaptation.
    /// Valid values: 0 < iter
    /// Defaults to 50
    pub iter: i32,
}
impl Default for VariationalAdapt {
    fn default() -> Self {
        VariationalAdaptBuilder::new().build()
    }
}

impl VariationalAdapt {
    pub fn command_fragment(&self) -> String {
        let mut s = String::from("adapt");
        write!(&mut s, " engaged={}", self.engaged as u8).unwrap();
        write!(&mut s, " iter={}", self.iter).unwrap();
        s
    }
    /// Return a builder with all options unspecified.
    pub fn builder() -> VariationalAdaptBuilder {
        VariationalAdaptBuilder::new()
    }
}

/// Options builder for `VariationalAdapt`.
/// For any option left unspecified, the default value indicated
/// on `VariationalAdapt` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct VariationalAdaptBuilder {
    engaged: Option<bool>,
    iter: Option<i32>,
}
impl VariationalAdaptBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            engaged: None,
            iter: None,
        }
    }
    insert_field!(engaged, bool);
    insert_field!(iter, i32);
    /// Build the `VariationalAdapt` instance.
    pub fn build(self) -> VariationalAdapt {
        let engaged = self.engaged.unwrap_or(true);
        let iter = self.iter.unwrap_or(50);
        VariationalAdapt { engaged, iter }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod variational {
        use super::*;

        #[test]
        fn builder() {
            let x = VariationalBuilder::new()
                .algorithm(VariationalAlgorithm::FullRank)
                .iter(1)
                .grad_samples(2)
                .elbo_samples(3)
                .eta(0.1)
                .adapt(VariationalAdapt::builder().engaged(false).iter(200).build())
                .tol_rel_obj(0.2)
                .eval_elbo(4)
                .output_samples(5)
                .build();
            assert_eq!(
                x,
                Method::Variational {
                    algorithm: VariationalAlgorithm::FullRank,
                    iter: 1,
                    grad_samples: 2,
                    elbo_samples: 3,
                    eta: 0.1,
                    adapt: VariationalAdapt::builder().engaged(false).iter(200).build(),
                    tol_rel_obj: 0.2,
                    eval_elbo: 4,
                    output_samples: 5
                }
            );

            let x = VariationalBuilder::new().build();
            assert_eq!(
                x,
                Method::Variational {
                    algorithm: VariationalAlgorithm::MeanField,
                    iter: 10000,
                    grad_samples: 1,
                    elbo_samples: 100,
                    eta: 1.0,
                    adapt: VariationalAdapt::default(),
                    tol_rel_obj: 0.01,
                    eval_elbo: 100,
                    output_samples: 1000,
                }
            );
        }
    }

    #[cfg(test)]
    mod algorithm {
        use super::*;

        #[test]
        fn default() {
            let x = VariationalAlgorithm::default();
            assert_eq!(x, VariationalAlgorithm::MeanField);
        }

        #[test]
        fn command_fragment() {
            let x = VariationalAlgorithm::default();
            assert_eq!(x.command_fragment(), "algorithm=meanfield");
            let x = VariationalAlgorithm::FullRank;
            assert_eq!(x.command_fragment(), "algorithm=fullrank");
        }
    }

    #[cfg(test)]
    mod adapt {
        use super::*;

        #[test]
        fn default() {
            let x = VariationalAdapt::default();
            assert_eq!(
                x,
                VariationalAdapt {
                    engaged: true,
                    iter: 50
                }
            );
        }

        #[test]
        fn builder() {
            let x = VariationalAdaptBuilder::new()
                .engaged(false)
                .iter(200)
                .build();
            assert_eq!(x.engaged, false);
            assert_eq!(x.iter, 200);
        }

        #[test]
        fn command_fragment() {
            let x = VariationalAdapt::default();
            assert_eq!(x.command_fragment(), "adapt engaged=1 iter=50");
            let x = VariationalAdaptBuilder::new()
                .engaged(false)
                .iter(200)
                .build();
            assert_eq!(x.command_fragment(), "adapt engaged=0 iter=200");
        }
    }
}
