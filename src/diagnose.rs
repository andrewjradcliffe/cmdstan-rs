#[derive(Debug)]
pub enum DiagnosticTest {
    /// Check model gradient against finite differences
    Gradient {
        /// Finite difference step size
        /// Valid values: 0 < epsilon
        /// Defaults to 1e-6
        epsilon: f64,
        /// Error threshold
        /// Valid values: 0 < error
        /// Defaults to 1e-6
        error: f64,
    },
}
impl Default for DiagnosticTest {
    fn default() -> Self {
        DiagnosticTest::Gradient {
            epsilon: 1e-6,
            error: 1e-6,
        }
    }
}
