use std::fmt::Write;

/// Diagnostic test
/// Valid values: gradient
/// Defaults to gradient
#[derive(Debug, PartialEq, Clone)]
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

impl DiagnosticTest {
    pub fn command_fragment(&self) -> String {
        match &self {
            DiagnosticTest::Gradient { epsilon, error } => {
                let mut s = String::from("test=gradient");
                write!(&mut s, " epsilon={}", epsilon).unwrap();
                write!(&mut s, " error={}", error).unwrap();
                s
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let x = DiagnosticTest::default();
        let y = DiagnosticTest::Gradient {
            epsilon: 1e-6_f64,
            error: 1e-6_f64,
        };
        assert_eq!(x, y);
    }

    #[test]
    fn command_fragment() {
        let x = DiagnosticTest::default();
        assert_eq!(
            x.command_fragment(),
            "test=gradient epsilon=0.000001 error=0.000001"
        );
    }
}
