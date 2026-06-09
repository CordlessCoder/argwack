#![no_std]
#![deny(clippy::missing_safety_doc)]
use alloc::{string::String, vec::Vec};
use rustc_hash::FxHashMapSeed;
use soa_rs::Soa;
use thiserror::Error;
extern crate alloc;

mod arg;
use crate::{
    arg::ArgContext,
    source::{AnyArgSource, ArgSegment},
};

mod help;
mod source;
pub use crate::arg::{Arg, ArgOut};
pub use crate::source::ArgSource;
pub use help::HelpMessage;
// use source::*;

pub mod prelude {
    // use str::FromStr;

    pub use crate::{ArgError, ArgOut, ArgSource, Arguments, arg::Arg};
    //
    // #[inline(always)]
    // pub fn opt_from_str<'s, T: FromStr>() -> Arg<'s, OptFromStrWrapper<T>>
    // where
    //     OptFromStrWrapper<T>: ArgumentValue<'s>,
    // {
    //     Arg::new(OptFromStrWrapper::NotFound)
    // }
    // #[inline(always)]
    // pub fn opt_none<'s, T>() -> Arg<'s, Option<T>>
    // where
    //     Option<T>: ArgumentValue<'s>,
    // {
    //     Arg::new(None)
    // }
    // #[inline(always)]
    // pub fn opt_by_ref<'m, 's, T: ArgumentValue<'s>>(v: &'m mut T) -> Arg<'s, SetViaRef<'m, T>>
    // where
    //     's: 'm,
    // {
    //     Arg::new(SetViaRef(v))
    // }
}

#[derive(Debug, Clone, Error)]
pub enum ArgError<'s> {
    #[error("Invalid value({1}) for parameter {0}")]
    InvalidValueForOpt(ArgContext, &'s str),
    #[error("Missing value for parameter {0}")]
    MissingValueForOpt(ArgContext),
    #[error("Unkown short option: {0}")]
    UnknownShortOption(char),
    #[error("Unkown long option: {0}")]
    UnknownLongOption(&'s str),
    #[error("{0}")]
    UnexpectedPositionalOpt(&'s str),
    #[error("{0}")]
    Custom(String),
}

#[must_use]
pub struct Arguments<'o, 'i, S> {
    pub args: Soa<Arg<'o, 'i>>,
    sink: S,
    pub program_name: Option<&'static str>,
    // MAX indicates empty slot
    short_lut: [usize; 256],
    long_map: FxHashMapSeed<&'static str, usize>,
}

impl Arguments<'_, '_, ()> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::new_with_sink(())
    }
}

impl Default for Arguments<'static, 'static, ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Arguments<'_, '_, S> {
    #[inline]
    pub fn new_with_sink(sink: S) -> Self {
        Self {
            args: Soa::new(),
            sink,
            program_name: None,
            short_lut: [usize::MAX; _],
            long_map: FxHashMapSeed::with_hasher(rustc_hash::FxSeededState::with_seed(12345)),
        }
    }
}

impl<'o, 'e> Arguments<'o, 'e, PositionalSink<'o, 'e>> {
    #[inline]
    pub fn new_with_positional(positional: Vec<ArgOut<'o, 'e>>) -> Self {
        Self {
            args: Soa::new(),
            sink: PositionalSink {
                idx: 0,
                args: positional,
            },
            program_name: None,
            short_lut: [usize::MAX; _],
            long_map: FxHashMapSeed::with_hasher(rustc_hash::FxSeededState::with_seed(12345)),
        }
    }
}

impl<'out, 'ext, S: ArgumentSink<'ext>> Arguments<'out, 'ext, S> {
    pub fn add(&mut self, arg: Arg<'out, 'ext>) -> &mut Self {
        let idx = self.args.len();
        if let Some(short) = &arg.ctx.short {
            let lookup_at = short.get();
            self.short_lut[lookup_at as usize] = idx;
        }
        if let Some(long) = arg.ctx.long {
            self.long_map.insert(long, idx);
        }
        self.args.push(arg);
        self
    }
    pub fn parse(&mut self, source: &mut impl AnyArgSource<'ext>) -> Result<(), ArgError<'ext>> {
        while let Some(segment) = source.next() {
            match segment {
                ArgSegment::Short(short) => {
                    let idx = self.short_lut[short as usize];
                    if idx == usize::MAX {
                        return Err(ArgError::UnknownShortOption(short as char));
                    }
                    // SAFETY: An index may only be inserted into self.short_lut if there is
                    // already an element there, and elements are never removed.
                    let arg = unsafe { self.args.get_mut(idx).unwrap_unchecked() };
                    arg.out.capture(arg.ctx, source)?;
                }
                ArgSegment::Long(long) => {
                    let Some(&idx) = self.long_map.get(long) else {
                        return Err(ArgError::UnknownLongOption(long));
                    };
                    // SAFETY: An index may only be inserted into self.long_map if there is
                    // already an element there, and elements are never removed.
                    let arg = unsafe { self.args.get_mut(idx).unwrap_unchecked() };
                    arg.out.capture(arg.ctx, source)?;
                }
                ArgSegment::Value(val) => {
                    self.sink.consume_value(val, source)?;
                }
            }
        }
        Ok(())
    }
}

pub trait ArgumentSink<'s> {
    fn consume_value(
        &mut self,
        value: &'s str,
        rest: &mut impl AnyArgSource<'s>,
    ) -> Result<(), ArgError<'s>>;
}

pub struct SubcommandSink;

impl<'s> ArgumentSink<'s> for SubcommandSink {
    fn consume_value(
        &mut self,
        value: &'s str,
        _rest: &mut impl AnyArgSource<'s>,
    ) -> Result<(), ArgError<'s>> {
        Err(ArgError::UnexpectedPositionalOpt(value))
    }
}

pub struct PositionalSink<'o, 'e> {
    idx: usize,
    args: Vec<ArgOut<'o, 'e>>,
}

impl<'s> ArgumentSink<'s> for PositionalSink<'_, 's> {
    fn consume_value(
        &mut self,
        value: &'s str,
        rest: &mut impl AnyArgSource<'s>,
    ) -> Result<(), ArgError<'s>> {
        if self.idx >= self.args.len() {
            return Err(ArgError::UnexpectedPositionalOpt(value));
        }
        let arg = &mut self.args[self.idx];
        arg.capture(&ArgContext::empty(), rest)?;
        self.idx += 1;
        Ok(())
    }
}

impl<'s> ArgumentSink<'s> for () {
    #[inline(always)]
    fn consume_value(
        &mut self,
        _value: &'s str,
        _rest: &mut impl AnyArgSource<'s>,
    ) -> Result<(), ArgError<'s>> {
        Ok(())
    }
}

impl<'s> ArgumentSink<'s> for alloc::vec::Vec<&'s str> {
    #[inline(always)]
    fn consume_value(
        &mut self,
        value: &'s str,
        _rest: &mut impl AnyArgSource<'s>,
    ) -> Result<(), ArgError<'s>> {
        self.push(value);
        Ok(())
    }
}

impl<'s, C: FnMut(&'s str) -> Result<(), ArgError<'s>>> ArgumentSink<'s> for C {
    #[inline(always)]
    fn consume_value(
        &mut self,
        value: &'s str,
        _rest: &mut impl AnyArgSource<'s>,
    ) -> Result<(), ArgError<'s>> {
        self(value)
    }
}

pub fn test_helper<'a>(name: &'static str, input: &[&'a str]) -> Result<(), ArgError<'a>> {
    let [mut q, mut w, mut e, mut r, mut t, mut y] = [false; _];
    let [mut u, mut i, mut o, mut p, mut a, mut s] = [None::<i64>; _];
    let [mut d, mut f, mut g, mut h, mut j, mut k] = [None::<f64>; _];
    let [mut l, mut z, mut x, mut c, mut v, mut b] = [None::<&str>; _];
    let mut args: Arguments<'_, '_, ()> = Arguments::new();
    args.program_name = Some(name);
    args
        // Bools
        .add(Arg::new(&mut q).with_short(b'q'))
        .add(Arg::new(&mut w).with_short(b'w'))
        .add(Arg::new(&mut e).with_short(b'e'))
        .add(Arg::new(&mut r).with_long(&"r"))
        .add(Arg::new(&mut t).with_long(&"t"))
        .add(Arg::new(&mut y).with_long(&"y"))
        // Large ints
        .add(Arg::new(&mut u).with_short(b'u'))
        .add(Arg::new(&mut i).with_short(b'i'))
        .add(Arg::new(&mut o).with_short(b'o'))
        .add(Arg::new(&mut p).with_long(&"p"))
        .add(Arg::new(&mut a).with_long(&"a"))
        .add(Arg::new(&mut s).with_long(&"s"))
        // Lots of floats
        .add(Arg::new(&mut d).with_short(b'd'))
        .add(Arg::new(&mut f).with_short(b'f'))
        .add(Arg::new(&mut g).with_short(b'g'))
        .add(Arg::new(&mut h).with_long(&"h"))
        .add(Arg::new(&mut j).with_long(&"j"))
        .add(Arg::new(&mut k).with_long(&"k"))
        // Stringy cheese
        .add(Arg::new(&mut l).with_short(b'l'))
        .add(Arg::new(&mut z).with_short(b'z'))
        .add(Arg::new(&mut x).with_short(b'x'))
        .add(Arg::new(&mut c).with_long(&"c"))
        .add(Arg::new(&mut v).with_long(&"v"))
        .add(Arg::new(&mut b).with_long(&"b"));

    for _ in 0..100_000_000 {
        args.parse(&mut ArgSource::new(input.iter().copied()))?;
    }
    Ok(())
}
