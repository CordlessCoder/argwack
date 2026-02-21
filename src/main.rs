#![expect(clippy::should_implement_trait)]
use std::{marker::PhantomData, num::NonZeroU8};

pub trait ArgumentValue<'s>: Sized {
    fn from_val(val: &'s str) -> Option<Self>;
}

impl ArgumentValue<'_> for u32 {
    fn from_val(val: &str) -> Option<Self> {
        val.parse().ok()
    }
}

impl<'s> ArgumentValue<'s> for &'s str {
    fn from_val(val: &'s str) -> Option<&'s str> {
        Some(val)
    }
}

pub trait ArgumentList {}

struct Empty;
impl ArgumentList for Empty {}

#[derive(Debug, Default)]
struct Arg<'s, T: ArgumentValue<'s>> {
    short: Option<NonZeroU8>,
    long: Option<&'static str>,
    help: Option<&'static str>,
    out: Option<T>,
    __phantom: PhantomData<&'s T>
}
impl<'s, T: ArgumentValue<'s>> ArgumentList for Arg<'s, T> {}

struct More<'s, T: ArgumentValue<'s>, A: ArgumentList> {
    rest: A,
    arg: Arg<'s, T>,
}
impl<'s, T: ArgumentValue<'s>, A: ArgumentList> ArgumentList for More<'s, T, A> {}

pub struct Arguments<A: ArgumentList> {
    args: A,
}

impl Arguments<Empty> {
    pub fn new() -> Self {
        Self { args: Empty }
    }
    pub fn add<'s, T: ArgumentValue<'s>>(self, argument: Arg<'s, T>) -> Arguments<Arg<'s, T>> {
        Arguments { args: argument }
    }
}

impl<'s, T: ArgumentValue<'s>> Arguments<Arg<'s, T>> {
    pub fn add<O: ArgumentValue<'s>>(self, argument: Arg<'s, O>) -> Arguments<More<'s, O, Arg<'s, T>>> {
        Arguments {
            args: More {
                rest: self.args,
                arg: argument,
            },
        }
    }
}

impl<'s, T: ArgumentValue<'s>, A: ArgumentList> Arguments<More<'s, T, A>> {
    pub fn add<O: ArgumentValue<'s>>(self, argument: Arg<'s, O>) -> Arguments<More<'s, O, More<'s, T, A>>> {
        Arguments {
            args: More {
                rest: self.args,
                arg: argument,
            },
        }
    }
}

impl Default for Arguments<Empty> {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let args = Arguments::new()
        .add(Arg {
            short: Some(b'a'.try_into().unwrap()),
            out: None::<u32>,
            ..Default::default()
        })
        .add(Arg {
            short: Some(b'b'.try_into().unwrap()),
            out: None::<u32>,
            ..Default::default()
        })
        .add(Arg {
            short: Some(b'c'.try_into().unwrap()),
            out: None::<u32>,
            ..Default::default()
        })
        .add(Arg {
            short: Some(b'c'.try_into().unwrap()),
            out: None::<&str>,
            ..Default::default()
        });
}
