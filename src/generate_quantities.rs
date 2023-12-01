use crate::method::Method;
use std::ffi::OsString;

/// Options builder for [`Method::GenerateQuantities`].
/// For any option left unspecified, the default value indicated
/// on `Method::GenerateQuantities` will be supplied.
#[derive(Debug, PartialEq, Clone)]
pub struct GenerateQuantitiesBuilder {
    fitted_params: Option<OsString>,
}
impl GenerateQuantitiesBuilder {
    /// Return a builder with all options unspecified.
    pub fn new() -> Self {
        Self {
            fitted_params: None,
        }
    }
    insert_into_field!(fitted_params, OsString);
    /// Build the `Method::GenerateQuantities` instance.
    pub fn build(self) -> Method {
        let fitted_params = self.fitted_params.unwrap_or_else(|| "".into());
        Method::GenerateQuantities { fitted_params }
    }
}
impl Default for GenerateQuantitiesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let x = GenerateQuantitiesBuilder::new()
            .fitted_params("big.csv")
            .build();
        assert_eq!(
            x,
            Method::GenerateQuantities {
                fitted_params: "big.csv".into()
            }
        );

        let x = GenerateQuantitiesBuilder::new().build();
        assert_eq!(
            x,
            Method::GenerateQuantities {
                fitted_params: "".into()
            }
        );
    }
}
