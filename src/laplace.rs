use crate::method::Method;
use std::ffi::OsString;

/// Options builder for [`Method::Laplace`].
/// For any option left unspecified, the default value indicated
/// on `Method::Laplace` will be supplied.
#[derive(Debug, Clone, PartialEq)]
pub struct LaplaceBuilder {
    mode: Option<OsString>,
    jacobian: Option<bool>,
    draws: Option<i32>,
}
impl LaplaceBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            mode: None,
            jacobian: None,
            draws: None,
        }
    }
    insert_into_field!(mode, OsString);
    insert_field!(jacobian, bool);
    insert_field!(draws, i32);
    /// Build the `Method::Laplace` instance.
    pub fn build(self) -> Method {
        let mode = self.mode.unwrap_or_else(|| "".into());
        let jacobian = self.jacobian.unwrap_or(true);
        let draws = self.draws.unwrap_or(1000);
        Method::Laplace {
            mode,
            jacobian,
            draws,
        }
    }
}

impl Default for LaplaceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = LaplaceBuilder::new()
            .mode("theta.json")
            .jacobian(false)
            .draws(10)
            .build();
        assert_eq!(
            x,
            Method::Laplace {
                mode: "theta.json".into(),
                jacobian: false,
                draws: 10
            }
        );
        let x = LaplaceBuilder::new().build();
        assert_eq!(
            x,
            Method::Laplace {
                mode: "".into(),
                jacobian: true,
                draws: 1000
            }
        );
    }
}
