use crate::method::Method;
use std::ffi::OsString;

/// Options builder for [`Method::LogProb`].
/// For any option left unspecified, the default value indicated
/// on `Method::LogProb` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct LogProbBuilder {
    unconstrained_params: Option<OsString>,
    constrained_params: Option<OsString>,
    jacobian: Option<bool>,
}
impl LogProbBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            unconstrained_params: None,
            constrained_params: None,
            jacobian: None,
        }
    }
    insert_into_field!(unconstrained_params, OsString);
    insert_into_field!(constrained_params, OsString);
    insert_field!(jacobian, bool);
    /// Build the `Method::LogProb` instance.
    pub fn build(self) -> Method {
        let unconstrained_params = self.unconstrained_params.unwrap_or_else(|| "".into());
        let constrained_params = self.constrained_params.unwrap_or_else(|| "".into());
        let jacobian = self.jacobian.unwrap_or(true);
        Method::LogProb {
            unconstrained_params,
            constrained_params,
            jacobian,
        }
    }
}

impl Default for LogProbBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = LogProbBuilder::new()
            .unconstrained_params("unc.txt")
            .constrained_params("c.txt")
            .jacobian(false)
            .build();
        assert_eq!(
            x,
            Method::LogProb {
                unconstrained_params: "unc.txt".into(),
                constrained_params: "c.txt".into(),
                jacobian: false
            }
        );
        let x = LogProbBuilder::new().build();
        assert_eq!(
            x,
            Method::LogProb {
                unconstrained_params: "".into(),
                constrained_params: "".into(),
                jacobian: true
            }
        );
    }
}
