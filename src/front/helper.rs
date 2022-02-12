use std::str::{self, Utf8Error, FromStr};

pub type Bytes = [u8];

#[macro_export]
macro_rules! syntax {
    ($func_name: ident, $tag_string: literal, $output_token: expr) => {
        fn $func_name<'a>(s: &'a Bytes) -> IResult<&Bytes, Token> {
            map(tag($tag_string), |_| $output_token)(s)
        }
    };
}

pub fn concat_slice_vec(a: &Bytes, b: Vec<u8>) -> Vec<u8> {
    let mut result = a.to_vec();
    result.extend(&b);
    result
}

pub fn convert_vec_utf8(v: Vec<u8>) -> Result<String, Utf8Error> {
    let slice = v.as_slice();
    str::from_utf8(slice).map(|s| s.to_owned())
}

pub fn str_from_bytes(c: &Bytes) -> Result<&str, Utf8Error> {
    str::from_utf8(c)
}

pub fn str_to_from_str<F: FromStr>(c: &str) -> Result<F, F::Err> {
    FromStr::from_str(c)
}