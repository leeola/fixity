use {
    super::{Addr, Key, Path, Scalar, Value},
    nom::{
        branch::alt,
        bytes::complete::{escaped_transform, is_not, tag, tag_no_case},
        character::complete::digit1,
        combinator::{all_consuming, map, map_res, value},
        multi::separated_list1,
        sequence::preceded,
        IResult,
    },
};

impl Value {
    /// An experimental implementation to parse a [`Value`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(s: &str) -> Result<Self, Error> {
        let (_, scalar) = parse_scalar(s).map_err(|err| Error::InvalidValue(format!("{}", err)))?;
        Ok(scalar.into())
    }
}
impl Key {
    /// An experimental implementation to parse a [`Key`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(s: &str) -> Result<Self, Error> {
        let (_, scalar) = parse_scalar(s).map_err(|err| Error::InvalidKey(format!("{}", err)))?;
        Ok(scalar.into())
    }
}
impl Path {
    /// An experimental implementation to parse a [`Path`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(s: &str) -> Result<Self, Error> {
        let (_, path) = parse_path(s).map_err(|err| Error::InvalidPath(format!("{}", err)))?;
        Ok(path)
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to create Key from cli: `{0}`")]
    InvalidKey(String),
    #[error("unable to create Value from cli: `{0}`")]
    InvalidValue(String),
    #[error("unable to create Path from cli: `{0}`")]
    InvalidPath(String),
}
fn parse_uint32(input: &str) -> IResult<&str, u32> {
    map_res(all_consuming(digit1), |s| u32::from_str_radix(s, 10))(input)
}
fn parse_string(input: &str) -> IResult<&str, String> {
    Ok(("", input.to_owned()))
}
fn parse_addr(input: &str) -> IResult<&str, Addr> {
    // all addrs are alphanum currently, so we may as well
    // enforce it.
    map(all_consuming(digit1), Addr::from)(input)
}
fn parse_untyped_scalar(input: &str) -> IResult<&str, Scalar> {
    alt((
        map(parse_uint32, Scalar::Uint32),
        map(parse_string, Scalar::String),
    ))(input)
}
fn parse_typed_scalar(input: &str) -> IResult<&str, Scalar> {
    // TODO: check if it has a foo: prefix? Otherwise these are being checked
    // needlessly.
    alt((
        preceded(tag_no_case("u32:"), map(parse_uint32, Scalar::Uint32)),
        preceded(tag_no_case("str:"), map(parse_string, Scalar::String)),
        preceded(tag_no_case("addr:"), map(parse_addr, Scalar::Addr)),
    ))(input)
}
fn parse_scalar(input: &str) -> IResult<&str, Scalar> {
    all_consuming(alt((parse_typed_scalar, parse_untyped_scalar)))(input)
}
fn parse_path(input: &str) -> IResult<&str, Path> {
    all_consuming(map(
        separated_list1(
            tag("/"),
            map_res(
                escaped_transform(is_not("/\\"), '\\', value("/", tag("/"))),
                |s: String| match parse_scalar(s.as_str()) {
                    Ok((_, scalar)) => Ok(Key::from(scalar)),
                    // Error should be impossible, since any string is a valid scalar
                    // as a last resort.
                    Err(_) => Err(()),
                },
            ),
        ),
        Path::new,
    ))(input)
}
#[cfg(test)]
pub mod test {
    use super::*;
    #[test]
    fn untyped_scalar() {
        assert_eq!(
            parse_untyped_scalar("foo"),
            Ok(("", Scalar::String("foo".into()))),
        );
        assert_eq!(parse_untyped_scalar("5"), Ok(("", Scalar::Uint32(5))));
        assert_eq!(parse_untyped_scalar("505"), Ok(("", Scalar::Uint32(505))));
    }
    #[test]
    fn typed_scalar() {
        assert_eq!(
            parse_typed_scalar("str:foo"),
            Ok(("", Scalar::String("foo".into()))),
        );
        assert_eq!(parse_typed_scalar("u32:5"), Ok(("", Scalar::Uint32(5))));
    }
    #[test]
    fn scalar() {
        assert_eq!(parse_scalar("foo"), Ok(("", Scalar::String("foo".into()))),);
        assert_eq!(parse_scalar("5"), Ok(("", Scalar::Uint32(5))));
        assert_eq!(
            parse_scalar("str:foo"),
            Ok(("", Scalar::String("foo".into()))),
        );
        assert_eq!(parse_scalar("u32:5"), Ok(("", Scalar::Uint32(5))));
        assert_eq!(
            parse_scalar("foo/bar"),
            Ok(("", Scalar::String("foo/bar".into())))
        );
    }
    #[test]
    fn path() {
        assert_eq!(parse_path("5"), Ok(("", Path::from(5u32))),);
        assert_eq!(
            parse_path("5/foo"),
            Ok(("", Path::from(5u32).push_chain("foo"))),
        );
        assert_eq!(parse_path("foo\\/bar"), Ok(("", Path::from("foo/bar"))),);
        assert_eq!(parse_path("5\\/foo"), Ok(("", Path::from("5/foo"))),);
        assert_eq!(
            parse_path("5/foo/bar\\/"),
            Ok(("", Path::from(5u32).push_chain("foo").push_chain("bar/"))),
        );
    }
}
