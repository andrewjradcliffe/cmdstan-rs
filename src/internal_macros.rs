/// Helper macro to implement builder pattern.
macro_rules! insert_field {
    ($F:ident, $T:ident) => {
        /// Configure the named option with the given value.
        pub fn $F(self, $F: $T) -> Self {
            let mut me = self;
            let _ = me.$F.insert($F);
            me
        }
    };
}
