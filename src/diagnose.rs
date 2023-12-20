use crate::method::Method;
use std::ffi::OsString;

/// Options builder for [`Method::Diagnose`].
/// For any option left unspecified, the default value indicated
/// on `Method::Diagnose` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct DiagnoseBuilder {
    test: Option<DiagnoseTest>,
}
impl DiagnoseBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self { test: None }
    }
    insert_into_field!(test, DiagnoseTest);
    /// Build the `Method::Diagnose` instance.
    pub fn build(self) -> Method {
        let test = self.test.unwrap_or_default();
        Method::Diagnose { test }
    }
}
impl Default for DiagnoseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostic test. Defaults to `Gradient`.
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum DiagnoseTest {
    /// Check model gradient against finite differences
    #[non_exhaustive]
    Gradient {
        /// Finite difference step size.
        /// Valid values: `0 < epsilon`.
        /// Defaults to `1e-6`
        epsilon: f64,
        /// Error threshold.
        /// Valid values: `0 < error`.
        /// Defaults to `1e-6`.
        error: f64,
    },
}
impl Default for DiagnoseTest {
    fn default() -> Self {
        GradientBuilder::new().build()
    }
}

impl DiagnoseTest {
    pub fn command_fragment(&self) -> Vec<OsString> {
        match &self {
            Self::Gradient { epsilon, error } => {
                vec![
                    "test=gradient".into(),
                    format!("epsilon={}", epsilon).into(),
                    format!("error={}", error).into(),
                ]
            }
        }
    }
}

impl From<GradientBuilder> for DiagnoseTest {
    fn from(x: GradientBuilder) -> Self {
        x.build()
    }
}

/// Options builder for [`DiagnoseTest::Gradient`].
/// For any option left unspecified, the default value indicated
/// on `DiagnoseTest::Gradient` will be supplied.
#[derive(Debug, Clone, PartialEq)]
pub struct GradientBuilder {
    epsilon: Option<f64>,
    error: Option<f64>,
}
impl GradientBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        GradientBuilder {
            epsilon: None,
            error: None,
        }
    }
    insert_field!(epsilon, f64);
    insert_field!(error, f64);
    /// Build the `DiagnoseTest::Gradient` instance.
    pub fn build(self) -> DiagnoseTest {
        let epsilon = self.epsilon.unwrap_or(1e-6);
        let error = self.error.unwrap_or(1e-6);
        DiagnoseTest::Gradient { epsilon, error }
    }
}
impl Default for GradientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = DiagnoseBuilder::new()
            .test(DiagnoseTest::Gradient {
                epsilon: 1e-1,
                error: 1e-1,
            })
            .build();
        assert_eq!(
            x,
            Method::Diagnose {
                test: DiagnoseTest::Gradient {
                    epsilon: 1e-1,
                    error: 1e-1
                }
            }
        );
    }

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
    fn command_fragment() {
        let x = DiagnoseTest::default();
        assert_eq!(
            x.command_fragment(),
            vec!["test=gradient", "epsilon=0.000001", "error=0.000001"]
        );
    }
}
