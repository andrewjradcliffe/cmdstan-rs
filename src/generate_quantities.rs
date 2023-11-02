use crate::method::Method;

/// Options builder for `Method::GenerateQuantities`.
/// For any option left unspecified, the default value indicated
/// on `Method::GenerateQuantities` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct GenerateQuantitiesBuilder {
    fitted_params: Option<String>,
}
impl GenerateQuantitiesBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            fitted_params: None,
        }
    }
    insert_into_field!(fitted_params, String);
    /// Build the `Method::GenerateQuantities` instance.
    pub fn build(self) -> Method {
        let fitted_params = self.fitted_params.unwrap_or_else(|| "".to_string());
        Method::GenerateQuantities { fitted_params }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = GenerateQuantitiesBuilder::new()
            .fitted_params("big.csv".to_string())
            .build();
        assert_eq!(
            x,
            Method::GenerateQuantities {
                fitted_params: "big.csv".to_string()
            }
        );

        let x = GenerateQuantitiesBuilder::new().build();
        assert_eq!(
            x,
            Method::GenerateQuantities {
                fitted_params: "".to_string()
            }
        );
    }
}
