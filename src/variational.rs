use std::fmt::Write;

/// Variational inference algorithm
/// Valid values: meanfield, fullrank
/// Defaults to meanfield
#[derive(Debug, Default, PartialEq)]
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
/// Valid subarguments: engaged, iter
#[derive(Debug, PartialEq)]
pub struct VariationalAdapt {
    /// Boolean flag for eta adaptation.
    /// Valid values: [0, 1]
    /// Defaults to 1
    engaged: bool,
    /// Number of iterations for eta adaptation.
    /// Valid values: 0 < iter
    /// Defaults to 50
    iter: i32,
}
impl Default for VariationalAdapt {
    fn default() -> Self {
        Self {
            engaged: true,
            iter: 50,
        }
    }
}

impl VariationalAdapt {
    pub fn command_fragment(&self) -> String {
        let mut s = String::from("adapt");
        write!(&mut s, " engaged={}", self.engaged as u8).unwrap();
        write!(&mut s, " iter={}", self.iter).unwrap();
        s
    }
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
        fn command_fragment() {
            let x = VariationalAdapt::default();
            assert_eq!(x.command_fragment(), "adapt engaged=1 iter=50");
        }
    }
}
