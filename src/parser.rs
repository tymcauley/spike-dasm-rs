use nom::{
    bytes::complete::{tag, take_until, take_while_m_n},
    character::complete::char,
    combinator::map_res,
    sequence::{delimited, pair},
    IResult,
};

const SIGNPOST: &str = "DASM(";

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

/// Parses an 8-character string as a hexadecimal number into a `u32`.
///
/// The input can't contain underscores or a leading '0x'/'0X', but mixed-case characters are fine.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(hex_u32("0123abCD").unwrap(), ("", 0x123abcd));
/// ```
fn hex_u32(input: &str) -> IResult<&str, u32> {
    map_res(take_while_m_n(8, 8, is_hex_digit), |out: &str| {
        u32::from_str_radix(&str::replace(&out, "_", ""), 16)
    })(input)
}

/// Parses a single hex-formatted u32 from within the "DASM(...)" signpost.
///
/// Leading and trailing characters are preserved, but the signpost itself is discarded.
///
/// # Examples
///
/// ```
/// use spike_dasm_rs::parser;
/// assert_eq!(
///     parser::parse_value("leading[DASM(0123def0)]trailing"),
///     Ok(("]trailing", ("leading[", 0x123def0)))
/// );
/// assert!(parser::parse_value("nothing to see here").is_err());
/// ```
pub fn parse_value(input: &str) -> IResult<&str, (&str, u32)> {
    pair(
        take_until(SIGNPOST),
        delimited(tag(SIGNPOST), hex_u32, char(')')),
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_hex_strings() {
        use nom::combinator::all_consuming;

        // Not only is this input valid...
        assert!(hex_u32("12345678").is_ok());
        // ...but there's nothing left to parse after we're done.
        assert!(all_consuming(hex_u32)("12345678").is_ok());

        // On the other hand, this input is invalid...
        assert!(hex_u32("xyz").is_err());

        // ...and this input has some trailing characters that don't match.
        assert_eq!(hex_u32("1234abcdxyz456").unwrap(), ("xyz456", 0x1234abcd));

        // We support mixed case.
        assert!(all_consuming(hex_u32)("12aBcDeF").is_ok());

        // We don't support underscores...
        assert!(all_consuming(hex_u32)("12ab_cdef").is_err());

        // ...or leading '0x'/'0X'.
        assert!(all_consuming(hex_u32)("0x12ABCDEF").is_err());
        assert!(all_consuming(hex_u32)("0X12abcdef").is_err());

        // Example from `hex_u32` docstring.
        assert_eq!(hex_u32("0123abCD").unwrap(), ("", 0x123abcd));
    }

    #[test]
    fn parse_lines() {
        let begin = "foo";
        let end = "bar";
        let test_str = format!("{}{}fedcab10){}", begin, SIGNPOST, end);
        assert_eq!(parse_value(&test_str), Ok((end, (begin, 0xfedcab10))));
    }
}
