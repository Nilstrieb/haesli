use haesli_core::{
    error::ConException,
    methods::{FieldValue, Method, Table},
};
use rand::Rng;

use crate::error::TransError;

mod generated;
pub mod parse_helper;
#[cfg(test)]
mod tests;
pub mod write_helper;

pub use generated::*;

/// Parses the payload of a method frame into the method
pub fn parse_method(payload: &[u8]) -> Result<Method, TransError> {
    let nom_result = generated::parse::parse_method(payload);

    match nom_result {
        Ok(([], method)) => Ok(method),
        Ok((_, _)) => {
            Err(ConException::SyntaxError(vec!["could not consume all input".to_owned()]).into())
        }
        Err(nom::Err::Incomplete(_)) => {
            Err(ConException::SyntaxError(vec!["there was not enough data".to_owned()]).into())
        }
        Err(nom::Err::Failure(err) | nom::Err::Error(err)) => Err(err),
    }
}

/// Allows the creation of a random instance of that type
pub trait RandomMethod<R: Rng> {
    fn random(rng: &mut R) -> Self;
}

impl<R: Rng> RandomMethod<R> for String {
    fn random(rng: &mut R) -> Self {
        let n = rng.gen_range(0_u16..9999);
        format!("string{n}")
    }
}

impl<R: Rng, T: RandomMethod<R>> RandomMethod<R> for Vec<T> {
    fn random(rng: &mut R) -> Self {
        let len = rng.gen_range(1_usize..10);
        let mut vec = Vec::with_capacity(len);
        (0..len).for_each(|_| vec.push(RandomMethod::random(rng)));
        vec
    }
}

macro_rules! rand_random_method {
    ($($ty:ty),+) => {
        $(
             impl<R: Rng> RandomMethod<R> for $ty {
             fn random(rng: &mut R) -> Self {
                rng.gen()
            }
        })+
    };
}

rand_random_method!(bool, u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

impl<R: Rng> RandomMethod<R> for Table {
    fn random(rng: &mut R) -> Self {
        let len = rng.gen_range(0..3);
        (0..len)
            .map(|_| (String::random(rng), FieldValue::random(rng)))
            .collect()
    }
}

impl<R: Rng> RandomMethod<R> for FieldValue {
    fn random(rng: &mut R) -> Self {
        let index = rng.gen_range(0_u32..17);
        match index {
            0 => Self::Boolean(RandomMethod::random(rng)),
            1 => Self::ShortShortInt(RandomMethod::random(rng)),
            2 => Self::ShortShortUInt(RandomMethod::random(rng)),
            3 => Self::ShortInt(RandomMethod::random(rng)),
            4 => Self::ShortUInt(RandomMethod::random(rng)),
            5 => Self::LongInt(RandomMethod::random(rng)),
            6 => Self::LongUInt(RandomMethod::random(rng)),
            7 => Self::LongLongInt(RandomMethod::random(rng)),
            8 => Self::LongLongUInt(RandomMethod::random(rng)),
            9 => Self::Float(RandomMethod::random(rng)),
            10 => Self::Double(RandomMethod::random(rng)),
            11 => Self::ShortString(RandomMethod::random(rng)),
            12 => Self::LongString(RandomMethod::random(rng)),
            13 => Self::FieldArray(RandomMethod::random(rng)),
            14 => Self::Timestamp(RandomMethod::random(rng)),
            15 => Self::FieldTable(RandomMethod::random(rng)),
            16 => Self::Void,
            _ => unreachable!(),
        }
    }
}
