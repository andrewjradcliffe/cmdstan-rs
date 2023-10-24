/// Variational inference algorithm
/// Valid values: meanfield, fullrank
/// Defaults to meanfield
#[derive(Debug, Default)]
pub enum VariationalAlgorithm {
    /// mean-field approximation
    #[default]
    MeanField,
    /// full-rank covariance
    FullRank,
}

/// Eta Adaptation for Variational Inference
/// Valid subarguments: engaged, iter
#[derive(Debug)]
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
