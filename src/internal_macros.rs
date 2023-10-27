/// Helper macro to implement builder pattern.
macro_rules! insert_field {
    ($F:ident, $T:ident) => {
        /// Configure the named option with the given value.
        pub fn $F(mut self, $F: $T) -> Self {
            let _ = self.$F.insert($F);
            self
        }
    };
}

// macro_rules! insert_string_field {
//     ($F:ident) => {
//         /// Configure the named option with the given value.
//         pub fn $F(mut self, $F: &str) -> Self {
//             let _ = self.$F.insert($F.to_string());
//             self
//         }
//     };
// }
