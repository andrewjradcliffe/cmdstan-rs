use crate::builder::Builder;
use crate::translate::Translate;
use std::ffi::OsString;

/// Variational inference algorithm. Defaults to
/// [`VariationalAlgorithm::MeanField`].
#[derive(Debug, Default, PartialEq, Clone, Translate)]
#[non_exhaustive]
#[declare = "algorithm"]
pub enum VariationalAlgorithm {
    /// mean-field approximation
    #[default]
    MeanField,
    /// full-rank covariance
    FullRank,
}

/// Eta Adaptation for Variational Inference
/// (i.e. [`Method::Variational`][crate::method::Method::Variational]).
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "adapt"]
pub struct VariationalAdapt {
    /// Boolean flag for eta adaptation.
    /// Defaults to `true`.
    ///
    /// At command line, this presents as `false` => 0, `true` => 1,
    /// with valid values 0 or 1.
    #[defaults_to = true]
    pub engaged: bool,
    /// Number of iterations for eta adaptation.
    /// Valid values: `0 < iter`.
    /// Defaults to `50`.
    #[defaults_to = 50]
    pub iter: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod algorithm {
        use super::*;

        #[test]
        fn default() {
            let x = VariationalAlgorithm::default();
            assert_eq!(x, VariationalAlgorithm::MeanField);
        }

        #[test]
        fn to_args() {
            let x = VariationalAlgorithm::default();
            assert_eq!(x.to_args(), vec!["algorithm=meanfield"]);
            let x = VariationalAlgorithm::FullRank;
            assert_eq!(x.to_args(), vec!["algorithm=fullrank"]);
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
            assert!(!x.engaged);
            assert_eq!(x.iter, 200);
        }

        #[test]
        fn to_args() {
            let x = VariationalAdapt::default();
            assert_eq!(x.to_args(), vec!["adapt", "engaged=1", "iter=50"]);
            let x = VariationalAdaptBuilder::new()
                .engaged(false)
                .iter(200)
                .build();
            assert_eq!(x.to_args(), vec!["adapt", "engaged=0", "iter=200"]);
        }
    }
}
