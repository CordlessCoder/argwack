#![expect(clippy::should_implement_trait)]
use std::{fmt::Display, hint::black_box, marker::PhantomData, num::NonZeroU8, time::Instant};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ArgError<'s> {
    #[error("Invalid value({1}) for parameter {0}")]
    InvalidValueForOpt(ArgContext, &'s str),
    #[error("Missing value for parameter {0}")]
    MissingValueForOpt(ArgContext),
    #[error("Unkown short option: {}", (*.0) as char)]
    UnknownShortOption(u8),
    #[error("Unkown long option: {0}")]
    UnknownLongOption(&'s str),
}

pub fn new_opt<'s, T>() -> Arg<'s, Option<T>>
where
    Option<T>: ArgumentValue<'s>,
{
    Arg::new(None)
}

#[derive(Debug, Clone)]
enum Saved<'a> {
    Empty,
    Value(&'a str),
    Shorts(&'a [u8]),
}

#[derive(Debug, Clone)]
pub struct ArgSource<'slice, 'a> {
    args: &'slice [&'a str],
    saved: Saved<'a>,
}

impl<'slice, 'a> ArgSource<'slice, 'a> {
    pub fn new(args: &'slice [&'a str]) -> Self {
        Self {
            args,
            saved: Saved::Empty,
        }
    }
}

pub enum ArgSegment<'s> {
    Short(u8),
    Long(&'s str),
    Value(&'s str),
}

impl<'s> ArgSource<'_, 's> {
    pub fn next_value(&mut self) -> Option<&'s str> {
        match self.saved {
            Saved::Value(val) => {
                self.saved = Saved::Empty;
                return Some(val);
            }
            Saved::Shorts([]) => {
                self.saved = Saved::Empty;
            }
            Saved::Shorts(_) => {
                return None;
            }
            Saved::Empty => (),
        }
        let (&first, rest) = self.args.split_first()?;
        if first.starts_with('-') {
            return None;
        }
        self.args = rest;
        Some(first)
    }
    pub fn next(&mut self) -> Option<ArgSegment<'s>> {
        match self.saved {
            Saved::Empty => (),
            Saved::Value(val) => {
                self.saved = Saved::Empty;
                return Some(ArgSegment::Value(val));
            }
            Saved::Shorts([]) => {
                self.saved = Saved::Empty;
            }
            Saved::Shorts([first, rest @ ..]) => {
                self.saved = Saved::Shorts(rest);
                return Some(ArgSegment::Short(*first));
            }
        }
        let (&first, rest) = self.args.split_first()?;
        self.args = rest;
        match first.as_bytes() {
            [b'-', b'-', name @ ..] => {
                let mut name = name;
                if let Some(eq) = memchr::memchr(b'=', name) {
                    self.saved = Saved::Value(&first[2 + eq + 1..]);
                    name = &name[..eq];
                }
                Some(ArgSegment::Long(unsafe {
                    core::str::from_utf8_unchecked(name)
                }))
            }
            [b'-', short_name] => Some(ArgSegment::Short(*short_name)),
            [b'-', short_name, b'=', val @ ..] => {
                self.saved = Saved::Value(unsafe { core::str::from_utf8_unchecked(val) });
                Some(ArgSegment::Short(*short_name))
            }
            [b'-', short_name, more_shorts @ ..] => {
                self.saved = Saved::Shorts(more_shorts);
                Some(ArgSegment::Short(*short_name))
            }
            _ => Some(ArgSegment::Value(first)),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ArgContext {
    pub short: Option<NonZeroU8>,
    pub long: Option<&'static str>,
    pub help: Option<&'static str>,
}

impl Display for ArgContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let help = self.help.unwrap_or("[No help message]");
        if self.short.is_none() && self.long.is_none() {
            f.write_str("No flags set!")?;
        }
        if let Some(short) = self.short {
            write!(f, "-{} ", short.get() as char)?;
        }
        if let Some(long) = self.long {
            write!(f, "--{long} ")?;
        }
        f.write_str(help)
    }
}

impl ArgContext {
    pub const fn empty() -> Self {
        Self {
            short: None,
            long: None,
            help: None,
        }
    }
}

pub trait ArgumentValue<'s>: Sized {
    fn capture(
        &mut self,
        ctx: &ArgContext,
        source: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>>;
}

impl<'s> ArgumentValue<'s> for bool {
    fn capture(
        &mut self,
        _ctx: &ArgContext,
        _source: &mut ArgSource<'_, '_>,
    ) -> Result<(), ArgError<'s>> {
        *self = true;
        Ok(())
    }
}
impl<'s> ArgumentValue<'s> for u32 {
    fn capture(
        &mut self,
        _ctx: &ArgContext,
        _source: &mut ArgSource<'_, '_>,
    ) -> Result<(), ArgError<'s>> {
        *self += 1;
        Ok(())
    }
}
impl<'s> ArgumentValue<'s> for Option<&'s str> {
    fn capture(
        &mut self,
        ctx: &ArgContext,
        source: &'_ mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>> {
        let value = source
            .next_value()
            .ok_or(ArgError::MissingValueForOpt(*ctx))?;
        *self = Some(value);
        Ok(())
    }
}
impl<'s> ArgumentValue<'s> for Option<i64> {
    fn capture(
        &mut self,
        ctx: &ArgContext,
        source: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>> {
        let value = source
            .next_value()
            .ok_or(ArgError::MissingValueForOpt(*ctx))?;
        let parsed = value
            .parse()
            .ok()
            .ok_or(ArgError::InvalidValueForOpt(*ctx, value))?;
        *self = Some(parsed);
        Ok(())
    }
}

pub trait ArgumentList<'s> {
    fn capture_short_arg(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        name: u8,
    ) -> Result<bool, ArgError<'s>>;
    fn capture_long_arg(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        name: &str,
    ) -> Result<bool, ArgError<'s>>;
}

struct Empty;
impl<'s> ArgumentList<'s> for Empty {
    fn capture_short_arg(
        &mut self,
        _args: &mut ArgSource<'_, '_>,
        _name: u8,
    ) -> Result<bool, ArgError<'s>> {
        Ok(false)
    }
    fn capture_long_arg(
        &mut self,
        _args: &mut ArgSource<'_, '_>,
        _name: &str,
    ) -> Result<bool, ArgError<'s>> {
        Ok(false)
    }
}

#[derive(Debug, Default)]
pub struct Arg<'s, T: ArgumentValue<'s>> {
    ctx: ArgContext,
    out: T,
    _phantom: PhantomData<&'s T>,
}

impl<'s, T> Arg<'s, Option<T>>
where
    Option<T>: ArgumentValue<'s>,
{
    pub fn new_opt() -> Self {
        Self {
            ctx: ArgContext::empty(),
            out: None,
            _phantom: PhantomData,
        }
    }
}

impl Arg<'_, bool> {
    pub fn new_flag() -> Self {
        Arg::new(false)
    }
}

impl Arg<'_, u32> {
    pub fn new_count() -> Self {
        Arg::new(0)
    }
}

impl<'s, T: ArgumentValue<'s>> Arg<'s, T> {
    pub fn new(val: T) -> Self {
        Self {
            ctx: ArgContext::empty(),
            out: val,
            _phantom: PhantomData,
        }
    }
    pub fn with_short(mut self, short: u8) -> Self {
        self.ctx.short = short.try_into().ok();
        self
    }
    pub fn with_long(mut self, long: &'static str) -> Self {
        self.ctx.long = Some(long);
        self
    }
    pub fn with_help(mut self, help: &'static str) -> Self {
        self.ctx.help = Some(help);
        self
    }
}

impl<'s, T: ArgumentValue<'s>> ArgumentList<'s> for Arg<'s, T> {
    fn capture_short_arg(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        name: u8,
    ) -> Result<bool, ArgError<'s>> {
        let Some(short) = self.ctx.short else {
            return Ok(false);
        };
        if short.get() != name {
            return Ok(false);
        }
        self.out.capture(&self.ctx, args)?;
        Ok(true)
    }
    fn capture_long_arg(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        name: &str,
    ) -> Result<bool, ArgError<'s>> {
        let Some(long) = self.ctx.long else {
            return Ok(false);
        };
        if name != long {
            return Ok(false);
        }
        self.out.capture(&self.ctx, args)?;
        Ok(true)
    }
}

pub struct More<'s, T: ArgumentValue<'s>, A: ArgumentList<'s>> {
    rest: A,
    arg: Arg<'s, T>,
}
impl<'s, T: ArgumentValue<'s>, A: ArgumentList<'s>> ArgumentList<'s> for More<'s, T, A> {
    fn capture_short_arg(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        name: u8,
    ) -> Result<bool, ArgError<'s>> {
        if self.arg.capture_short_arg(args, name)? {
            return Ok(true);
        }
        self.rest.capture_short_arg(args, name)
    }
    fn capture_long_arg(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        name: &str,
    ) -> Result<bool, ArgError<'s>> {
        if self.arg.capture_long_arg(args, name)? {
            return Ok(true);
        }
        self.rest.capture_long_arg(args, name)
    }
}

pub struct Arguments<A, S> {
    args: A,
    sink: S,
}

impl Arguments<Empty, ()> {
    pub fn new() -> Self {
        Self {
            args: Empty,
            sink: (),
        }
    }
}
impl<S> Arguments<Empty, S> {
    pub fn new_with_sink(sink: S) -> Self {
        Self { args: Empty, sink }
    }
    pub fn add<'s, T: ArgumentValue<'s>>(self, argument: Arg<'s, T>) -> Arguments<Arg<'s, T>, S> {
        Arguments {
            args: argument,
            sink: self.sink,
        }
    }
}

impl<'s, T: ArgumentValue<'s>, S> Arguments<Arg<'s, T>, S> {
    pub fn add<O: ArgumentValue<'s>>(
        self,
        argument: Arg<'s, O>,
    ) -> Arguments<More<'s, O, Arg<'s, T>>, S> {
        let Self { args, sink } = self;
        Arguments {
            args: More {
                rest: args,
                arg: argument,
            },
            sink,
        }
    }
}

impl<'s, T: ArgumentValue<'s>, A: ArgumentList<'s>, S> Arguments<More<'s, T, A>, S> {
    pub fn add<O: ArgumentValue<'s>>(
        self,
        argument: Arg<'s, O>,
    ) -> Arguments<More<'s, O, More<'s, T, A>>, S> {
        Arguments {
            args: More {
                rest: self.args,
                arg: argument,
            },
            sink: self.sink,
        }
    }
}

impl Default for Arguments<Empty, ()> {
    fn default() -> Self {
        Self::new_with_sink(())
    }
}

pub trait ArgumentSink<'s> {
    fn consume_value(&mut self, value: &'s str) -> Result<(), ArgError<'s>>;
}

impl<'s> ArgumentSink<'s> for () {
    fn consume_value(&mut self, _value: &'s str) -> Result<(), ArgError<'s>> {
        Ok(())
    }
}

impl<'s> ArgumentSink<'s> for Vec<&'s str> {
    fn consume_value(&mut self, value: &'s str) -> Result<(), ArgError<'s>> {
        self.push(value);
        Ok(())
    }
}

impl<'s, C: FnMut(&'s str) -> Result<(), ArgError<'s>>> ArgumentSink<'s> for C {
    fn consume_value(&mut self, value: &'s str) -> Result<(), ArgError<'s>> {
        self(value)
    }
}

impl<'s, A: ArgumentList<'s>, S: ArgumentSink<'s>> Arguments<A, S> {
    pub fn with_sink<NS: ArgumentSink<'s>>(self, new_sink: NS) -> Arguments<A, NS> {
        let Self { args, sink: _sink } = self;
        Arguments {
            args,
            sink: new_sink,
        }
    }
    pub fn parse(&mut self, args: &[&'s str]) -> Result<(), ArgError<'s>> {
        let mut source = ArgSource::new(args);
        while let Some(segment) = source.next() {
            match segment {
                ArgSegment::Short(short) => {
                    if !self.args.capture_short_arg(&mut source, short)? {
                        return Err(ArgError::UnknownShortOption(short));
                    }
                }
                ArgSegment::Long(long) => {
                    if !self.args.capture_long_arg(&mut source, long)? {
                        return Err(ArgError::UnknownLongOption(long));
                    }
                }
                ArgSegment::Value(val) => {
                    self.sink.consume_value(val)?;
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let mut args = Arguments::new()
        // Bunch-a bools
        .add(Arg::new_flag().with_short(b'q'))
        .add(Arg::new_flag().with_short(b'w'))
        .add(Arg::new_flag().with_short(b'e'))
        .add(Arg::new_flag().with_long("r"))
        .add(Arg::new_flag().with_long("t"))
        .add(Arg::new_flag().with_long("y"))
        // LONG, LONG MAAAAAAAAAAN
        .add(new_opt::<i64>().with_short(b'u'))
        .add(new_opt::<i64>().with_short(b'i'))
        .add(new_opt::<i64>().with_short(b'o'))
        .add(new_opt::<i64>().with_long("p"))
        .add(new_opt::<i64>().with_long("a"))
        .add(new_opt::<i64>().with_long("s"))
        ;
    let start = Instant::now();
    for _ in 0..1_000_000 {
        args.parse(&["", "-q", "-q", "-q", "-q", "-q"]).unwrap();
        black_box(&args);
    }
    let took = start.elapsed();
    println!("1 million parses of 6 args each took {took:?}");
    // .add(Arg {
    //     short: Some(b'b'.try_into().unwrap()),
    //     out: None::<u32>,
    //     ..Default::default()
    // })
    // .add(Arg {
    //     short: Some(b'c'.try_into().unwrap()),
    //     out: None::<u32>,
    //     ..Default::default()
    // })
    // .add(Arg {
    //     short: Some(b'c'.try_into().unwrap()),
    //     out: None::<&str>,
    //     ..Default::default()
    // });
}
