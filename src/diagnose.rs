use crate::builder::Builder;
use crate::translate::Translate;
use std::ffi::OsString;

/// Diagnostic test. Defaults to [`DiagnoseTest::Gradient`].
#[derive(Debug, PartialEq, Clone, Translate, Builder)]
#[non_exhaustive]
#[declare = "test"]
pub enum DiagnoseTest {
    /// Check model gradient against finite differences
    #[non_exhaustive]
    Gradient {
        /// Finite difference step size.
        /// Valid values: `0 < epsilon`.
        /// Defaults to `1e-6`
        #[defaults_to = 0.000001]
        epsilon: f64,
        /// Error threshold.
        /// Valid values: `0 < error`.
        /// Defaults to `1e-6`.
        #[defaults_to = 0.000001]
        error: f64,
    },
}
impl Default for DiagnoseTest {
    fn default() -> Self {
        GradientBuilder::new().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let x = DiagnoseTest::default();
        let y = DiagnoseTest::Gradient {
            epsilon: 1e-6_f64,
            error: 1e-6_f64,
        };
        assert_eq!(x, y);
    }

    #[test]
    fn to_args() {
        let x = DiagnoseTest::default();
        assert_eq!(
            x.to_args(),
            vec!["test=gradient", "epsilon=0.000001", "error=0.000001"]
        );
    }
}
