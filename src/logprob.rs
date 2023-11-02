use crate::method::Method;

/// Options builder for `Method::LogProb`.
/// For any option left unspecified, the default value indicated
/// on `Method::LogProb` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct LogProbBuilder {
    unconstrained_params: Option<String>,
    constrained_params: Option<String>,
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
    insert_into_field!(unconstrained_params, String);
    insert_into_field!(constrained_params, String);
    insert_field!(jacobian, bool);
    /// Build the `Method::LogProb` instance.
    pub fn build(self) -> Method {
        let unconstrained_params = self.unconstrained_params.unwrap_or_else(|| "".to_string());
        let constrained_params = self.constrained_params.unwrap_or_else(|| "".to_string());
        let jacobian = self.jacobian.unwrap_or(true);
        Method::LogProb {
            unconstrained_params,
            constrained_params,
            jacobian,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = LogProbBuilder::new()
            .unconstrained_params("unc.txt".to_string())
            .constrained_params("c.txt".to_string())
            .jacobian(false)
            .build();
        assert_eq!(
            x,
            Method::LogProb {
                unconstrained_params: "unc.txt".to_string(),
                constrained_params: "c.txt".to_string(),
                jacobian: false
            }
        );
        let x = LogProbBuilder::new().build();
        assert_eq!(
            x,
            Method::LogProb {
                unconstrained_params: "".to_string(),
                constrained_params: "".to_string(),
                jacobian: true
            }
        );
    }
}
