use crate::method::Method;

/// Options builder for `Method::Laplace`.
/// For any option left unspecified, the default value indicated
/// on `Method::Laplace` will be supplied.
pub struct LaplaceBuilder {
    mode: Option<String>,
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
    insert_field!(mode, String);
    insert_field!(jacobian, bool);
    insert_field!(draws, i32);
    /// Build the `Method::Laplace` instance.
    pub fn build(self) -> Method {
        let mode = self.mode.unwrap_or_else(|| "".to_string());
        let jacobian = self.jacobian.unwrap_or(true);
        let draws = self.draws.unwrap_or(1000);
        Method::Laplace {
            mode,
            jacobian,
            draws,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = LaplaceBuilder::new()
            .mode("theta.json".to_string())
            .jacobian(false)
            .draws(10)
            .build();
        assert_eq!(
            x,
            Method::Laplace {
                mode: "theta.json".to_string(),
                jacobian: false,
                draws: 10
            }
        );
        let x = LaplaceBuilder::new().build();
        assert_eq!(
            x,
            Method::Laplace {
                mode: "".to_string(),
                jacobian: true,
                draws: 1000
            }
        );
    }
}
