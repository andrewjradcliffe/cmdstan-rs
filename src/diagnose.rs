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
    insert_field!(test, DiagnoseTest);
    /// Build the `Method::Diagnose` instance.
    pub fn build(self) -> Method {
        let test = self.test.unwrap_or_default();
        Method::Diagnose { test }
    }
}

/// Diagnostic test. Defaults to `Gradient`.
#[derive(Debug, PartialEq, Clone)]
pub enum DiagnoseTest {
    /// Check model gradient against finite differences
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
        DiagnoseTest::Gradient {
            epsilon: 1e-6,
            error: 1e-6,
        }
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
