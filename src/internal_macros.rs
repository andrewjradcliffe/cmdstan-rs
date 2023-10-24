macro_rules! insert_field {
    ($F:ident, $T:ident) => {
        pub fn $F(self, $F: $T) -> Self {
            let mut me = self;
            let _ = me.$F.insert($F);
            me
        }
    };
}
