/// Helper macro to implement builder pattern.
macro_rules! insert_field {
    ($F:ident, $T:ident) => {
        /// Configure the named option with the given value.
        pub fn $F(mut self, $F: $T) -> Self {
            self.$F = Some($F);
            self
        }
    };
}

macro_rules! insert_into_string_field {
    ($F:ident) => {
        /// Configure the named option with the given value.
        pub fn $F<T: Into<String>>(mut self, $F: T) -> Self {
            self.$F = Some($F.into());
            self
        }
    };
}

macro_rules! insert_into_field {
    ($F:ident, $U:ident) => {
        /// Configure the named option with the given value.
        pub fn $F<T: Into<$U>>(mut self, $F: T) -> Self {
            self.$F = Some($F.into());
            self
        }
    };
}
