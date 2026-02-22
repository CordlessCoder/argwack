#![expect(clippy::should_implement_trait)]
use std::marker::PhantomData;

use thiserror::Error;

mod arg;
use crate::arg::{Arg, ArgContext};

mod help;
mod source;
mod values;
pub use help::HelpMessage;
use source::*;
pub use values::{OptFromStrWrapper, SetViaRef};

pub mod prelude {
    use std::str::FromStr;

    pub use crate::{ArgError, Arguments, OptFromStrWrapper, arg::Arg};
    use crate::{ArgumentValue, values::SetViaRef};

    pub fn new_opt<'s, T: FromStr>() -> Arg<'s, OptFromStrWrapper<T>>
    where
        OptFromStrWrapper<T>: ArgumentValue<'s>,
    {
        Arg::new(OptFromStrWrapper::NotFound)
    }
    pub fn new_opt_none<'s, T>() -> Arg<'s, Option<T>>
    where
        Option<T>: ArgumentValue<'s>,
    {
        Arg::new(None)
    }
    pub fn opt_by_ref<'m, 's, T: ArgumentValue<'s>>(v: &'m mut T) -> Arg<'s, SetViaRef<'m, T>>
    where
        's: 'm,
    {
        Arg::new(SetViaRef(v))
    }
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

pub trait ArgumentValue<'s> {
    fn capture(
        &mut self,
        ctx: &ArgContext,
        source: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>>;
}

pub trait ArgumentList<'s> {
    type Values;

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
    fn into_values(self) -> Self::Values;
    /// Visit all [ArgContext]'s in index order.
    fn visit_ctxs<E>(&self, cb: &mut impl FnMut(&ArgContext) -> Result<(), E>) -> Result<(), E>;
    // TODO: Replace dynamic dispatch with contunuation-passing/capture-by-index
    /// Find the Arg corresponding to a given index
    fn lookup_index(&mut self, index: u8) -> Option<(ArgContext, &mut dyn ArgumentValue<'s>)>;
    fn len(&self) -> usize;
}

pub struct Empty;
impl<'s> ArgumentList<'s> for Empty {
    type Values = ();

    #[inline(always)]
    fn capture_short_arg(
        &mut self,
        _args: &mut ArgSource<'_, 's>,
        _name: u8,
    ) -> Result<bool, ArgError<'s>> {
        Ok(false)
    }
    #[inline(always)]
    fn capture_long_arg(
        &mut self,
        _args: &mut ArgSource<'_, 's>,
        _name: &str,
    ) -> Result<bool, ArgError<'s>> {
        Ok(false)
    }
    #[inline(always)]
    fn visit_ctxs<E>(&self, _cb: &mut impl FnMut(&ArgContext) -> Result<(), E>) -> Result<(), E> {
        Ok(())
    }

    #[inline(always)]
    fn into_values(self) -> Self::Values {}
    #[inline(always)]
    fn lookup_index(&mut self, _index: u8) -> Option<(ArgContext, &mut dyn ArgumentValue<'s>)> {
        None
    }
    #[inline(always)]
    fn len(&self) -> usize {
        0
    }
}

impl Arg<'_, bool> {
    #[inline(always)]
    pub fn new_flag() -> Self {
        Arg::new(false)
    }
}

impl Arg<'_, u32> {
    #[inline(always)]
    pub fn new_count() -> Self {
        Arg::new(0)
    }
}

impl<'s, T: ArgumentValue<'s> + Default> Arg<'s, T> {
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            ctx: ArgContext::empty(),
            out: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<'s, T: ArgumentValue<'s>> Arg<'s, T> {
    #[inline(always)]
    pub fn new(val: T) -> Self {
        Self {
            ctx: ArgContext::empty(),
            out: val,
            _phantom: PhantomData,
        }
    }
    #[inline(always)]
    pub fn with_short(mut self, short: u8) -> Self {
        self.ctx.short = short.try_into().ok();
        self
    }
    #[inline(always)]
    pub fn with_long(mut self, long: &'static str) -> Self {
        self.ctx.long = Some(long);
        self
    }
    #[inline(always)]
    pub fn with_help(mut self, help: &'static str) -> Self {
        self.ctx.help = Some(help);
        self
    }
}

impl<'s, T: ArgumentValue<'s>> ArgumentList<'s> for Arg<'s, T> {
    type Values = T;

    #[inline(always)]
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
    #[inline(always)]
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

    #[inline(always)]
    fn into_values(self) -> Self::Values {
        self.out
    }
    #[inline(always)]
    fn visit_ctxs<E>(&self, cb: &mut impl FnMut(&ArgContext) -> Result<(), E>) -> Result<(), E> {
        cb(&self.ctx)
    }
    #[inline(always)]
    fn lookup_index(&mut self, index: u8) -> Option<(ArgContext, &mut dyn ArgumentValue<'s>)> {
        if index != 0 {
            return None;
        }
        Some((self.ctx, &mut self.out))
    }
    #[inline(always)]
    fn len(&self) -> usize {
        1
    }
}

pub struct More<'s, T: ArgumentValue<'s>, A: ArgumentList<'s>> {
    pub rest: A,
    pub arg: Arg<'s, T>,
}
impl<'s, T: ArgumentValue<'s>, A: ArgumentList<'s>> ArgumentList<'s> for More<'s, T, A> {
    type Values = (A::Values, T);

    #[inline]
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
    #[inline]
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

    #[inline(always)]
    fn into_values(self) -> Self::Values {
        (self.rest.into_values(), self.arg.into_values())
    }
    #[inline(always)]
    fn visit_ctxs<E>(&self, cb: &mut impl FnMut(&ArgContext) -> Result<(), E>) -> Result<(), E> {
        self.rest.visit_ctxs(cb)?;
        cb(&self.arg.ctx)
    }
    #[inline(always)]
    fn lookup_index(&mut self, index: u8) -> Option<(ArgContext, &mut dyn ArgumentValue<'s>)> {
        if index == 0 {
            return Some((self.arg.ctx, &mut self.arg.out));
        }
        self.rest.lookup_index(index - 1)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        1 + self.rest.len()
    }
}

pub struct Arguments<A, S> {
    pub args: A,
    sink: S,
    program_name: Option<&'static str>,
    // TODO: Support for more than 255 options(use something other than u8)
    short_lut: [u8; 256],
}

const fn empty_lut() -> [u8; 256] {
    [u8::MAX; _]
}

fn add_to_lut(lut: &mut [u8; 256], ctx: &ArgContext) {
    // NOTE: Needs to be adjusted if the option limit is changed
    lut.iter_mut()
        .filter(|&&mut idx| idx != u8::MAX)
        .for_each(|idx| *idx += 1);
    let Some(short) = ctx.short else {
        return;
    };
    lut[short.get() as usize] = 0;
}

impl Arguments<Empty, ()> {
    pub fn new() -> Self {
        Self {
            args: Empty,
            sink: (),
            program_name: None,
            short_lut: empty_lut(),
        }
    }
}
impl<S> Arguments<Empty, S> {
    pub fn new_with_sink(sink: S) -> Self {
        Self {
            args: Empty,
            sink,
            program_name: None,
            short_lut: empty_lut(),
        }
    }
    pub fn add<'s, T: ArgumentValue<'s>>(self, argument: Arg<'s, T>) -> Arguments<Arg<'s, T>, S> {
        let Self {
            args: _,
            sink,
            program_name,
            mut short_lut,
        } = self;
        add_to_lut(&mut short_lut, &argument.ctx);
        Arguments {
            args: argument,
            sink,
            program_name,
            short_lut,
        }
    }
}

impl<'s, T: ArgumentValue<'s>, S> Arguments<Arg<'s, T>, S> {
    pub fn add<O: ArgumentValue<'s>>(
        self,
        argument: Arg<'s, O>,
    ) -> Arguments<More<'s, O, Arg<'s, T>>, S> {
        let Self {
            args,
            sink,
            program_name,
            mut short_lut,
        } = self;
        add_to_lut(&mut short_lut, &argument.ctx);
        Arguments {
            args: More {
                rest: args,
                arg: argument,
            },
            sink,
            program_name,
            short_lut,
        }
    }
}

impl<'s, T: ArgumentValue<'s>, A: ArgumentList<'s>, S> Arguments<More<'s, T, A>, S> {
    pub fn add<O: ArgumentValue<'s>>(
        self,
        argument: Arg<'s, O>,
    ) -> Arguments<More<'s, O, More<'s, T, A>>, S> {
        let Self {
            args,
            sink,
            program_name,
            mut short_lut,
        } = self;
        let len = args.len();
        // NOTE: Needs to be relaxed if the argument count limit is increased
        assert!(len < 255);
        add_to_lut(&mut short_lut, &argument.ctx);
        Arguments {
            args: More {
                rest: args,
                arg: argument,
            },
            sink,
            program_name,
            short_lut,
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
        let Self {
            args,
            sink: _sink,
            program_name,
            short_lut,
        } = self;
        Arguments {
            args,
            sink: new_sink,
            program_name,
            short_lut,
        }
    }
    pub fn with_program_name(mut self, name: &'static str) -> Self {
        self.program_name = Some(name);
        self
    }
    pub fn parse(&mut self, args: &[&'s str]) -> Result<(), ArgError<'s>> {
        let mut source = ArgSource::new(args);
        while let Some(segment) = source.next() {
            match segment {
                ArgSegment::Short(short) => {
                    let idx = self.short_lut[short as usize];
                    if idx == 255 {
                        return Err(ArgError::UnknownShortOption(short as char));
                    }
                    let (ctx, val) = self.args.lookup_index(idx).unwrap();
                    val.capture(&ctx, &mut source)?;
                    debug_assert!(
                        ctx.short.is_some_and(|s| s.get() == short),
                        "{:?} must be equal to {:?}",
                        ctx.short.map(|v| v.get() as char),
                        short as char
                    );
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
    pub fn into_values(self) -> (A::Values, S) {
        let Self {
            args,
            sink,
            program_name: _,
            short_lut: _,
        } = self;
        (args.into_values(), sink)
    }
    pub fn help_msg<'a>(&'a self) -> HelpMessage<'a, A, S> {
        HelpMessage(self)
    }
}
