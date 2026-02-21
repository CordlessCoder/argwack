#![expect(clippy::should_implement_trait)]
use std::marker::PhantomData;

use thiserror::Error;

mod arg;
use crate::arg::{Arg, ArgContext};

mod source;
mod values;
use source::*;

pub mod prelude {
    pub use crate::{ArgError, arg::Arg, new_opt, Arguments};
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
}

pub fn new_opt<'s, T>() -> Arg<'s, Option<T>>
where
    Option<T>: ArgumentValue<'s>,
{
    Arg::new(None)
}

pub trait ArgumentValue<'s>: Sized {
    fn capture(
        &mut self,
        ctx: &ArgContext,
        source: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>>;
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

pub struct Empty;
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
                        return Err(ArgError::UnknownShortOption(short as char));
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
