pub mod de;
pub mod type_name {
    // TODO: Make a derive for this.
    // TODO: Ideally this would be const, though this is not stable yet.
    // Tracking issue(s) of interest:
    // - https://github.com/rust-lang/rust/issues/67792
    pub trait TypeName {
        fn type_name() -> String {
            todo!()
        }
        fn self_type() -> &'static str;
        fn param_types() -> &'static [&'static str];
    }
    // impl TypeName for
    // TODO: Ideally this would be const, though this is not stable yet.
    // Tracking issue(s) of interest:
    // - https://github.com/rust-lang/rust/issues/67792
    pub trait NamableType {
        fn self_type() -> &'static str;
        fn param_types() -> &'static [&'static str];
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
