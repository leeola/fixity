use {
    super::{Addr, Key, Path, Scalar, Value},
    nom::{
        branch::alt,
        bytes::complete::{escaped, tag, tag_no_case, take_until},
        character::complete::{alphanumeric1, digit1, one_of},
        combinator::{map, map_res},
        multi::separated_list1,
        sequence::preceded,
        IResult,
    },
};

impl Scalar {
    /// An experimental implementation to parse a [`Scalar`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(__s: &str) -> Result<Self, Error> {
        todo!("Scalar from cli str")
    }
}
impl Value {
    /// An experimental implementation to parse a [`Value`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(__s: &str) -> Result<Self, Error> {
        todo!("Value from cli str")
    }
}
impl Key {
    /// An experimental implementation to parse a [`Key`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(__s: &str) -> Result<Self, Error> {
        todo!("Key from cli str")
    }
}
impl Path {
    /// An experimental implementation to parse a [`Path`] value from a string
    /// focused interface; eg parsing values from the command line.
    ///
    /// This differs from a `FromStr` implementation in that there may be multiple
    /// interfaces tailored towards different user interfaces.
    pub fn from_cli_str(__s: &str) -> Result<Self, Error> {
        todo!("Path from cli str")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid u32: `{0}`")]
    InvalidUint32(String),
}

fn parse_uint32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s| u32::from_str_radix(s, 10))(input)
}
fn parse_string(input: &str) -> IResult<&str, String> {
    Ok(("", input.to_owned()))
}
fn parse_addr(input: &str) -> IResult<&str, Addr> {
    // all addrs are alphanum currently, so we may as well
    // enforce it.
    map(alphanumeric1, Addr::from)(input)
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
    alt((parse_typed_scalar, parse_untyped_scalar))(input)
}
fn scalar<'a, F>(mut f: F) -> impl FnMut(&'a str) -> IResult<&'a str, Scalar>
where
    F: FnMut(&'a str) -> IResult<&'a str, &'a str>,
{
    move |input| {
        let (input, scalar_input) = f(input)?;
        let (_, scalar) = alt((parse_typed_scalar, parse_untyped_scalar))(scalar_input)?;
        Ok((input, scalar))
    }
}
fn parse_path(input: &str) -> IResult<&str, Path> {
    map(
        separated_list1(
            tag("/"),
            scalar(escaped(take_until("/"), '\\', one_of("/"))),
        ),
        |v: Vec<Scalar>| Path::new(v.into_iter().map(Key::from).collect()),
    )(input)
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
    }
}
