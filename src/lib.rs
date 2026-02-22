#![expect(clippy::should_implement_trait)]
use std::marker::PhantomData;

use rustc_hash::FxHashMap;
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

    #[inline(always)]
    pub fn opt_from_str<'s, T: FromStr>() -> Arg<'s, OptFromStrWrapper<T>>
    where
        OptFromStrWrapper<T>: ArgumentValue<'s>,
    {
        Arg::new(OptFromStrWrapper::NotFound)
    }
    #[inline(always)]
    pub fn opt_none<'s, T>() -> Arg<'s, Option<T>>
    where
        Option<T>: ArgumentValue<'s>,
    {
        Arg::new(None)
    }
    #[inline(always)]
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

#[expect(clippy::len_without_is_empty)]
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
    fn capture_by_index(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        index: u16,
    ) -> Result<(), ArgError<'s>>;
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
    fn capture_by_index(
        &mut self,
        _args: &mut ArgSource<'_, 's>,
        _index: u16,
    ) -> Result<(), ArgError<'s>> {
        unreachable!()
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
    fn capture_by_index(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        index: u16,
    ) -> Result<(), ArgError<'s>> {
        if index != 0 {
            unreachable!()
        }
        self.out.capture(&self.ctx, args)
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
    fn capture_by_index(
        &mut self,
        args: &mut ArgSource<'_, 's>,
        index: u16,
    ) -> Result<(), ArgError<'s>> {
        if index == 0 {
            return self.arg.out.capture(&self.arg.ctx, args);
        }
        self.rest.capture_by_index(args, index - 1)
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
    // NOTE: Intentionally 1 less than [u8::MAX], as 0 isn't a valid shorthand
    short_lut: [u16; 255],
    long_map: FxHashMap<&'static str, u16>,
}

const fn empty_lut() -> [u16; 255] {
    [u16::MAX; _]
}

fn add_to_lut(lut: &mut [u16; 255], ctx: &ArgContext) {
    lut.iter_mut()
        .filter(|&&mut idx| idx != u16::MAX)
        .for_each(|idx| *idx += 1);
    let Some(short) = ctx.short else {
        return;
    };
    lut[short.get().wrapping_sub(1) as usize] = 0;
}

fn add_to_map(map: &mut FxHashMap<&'static str, u16>, ctx: &ArgContext) {
    map.values_mut().for_each(|idx| *idx += 1);
    if let Some(long) = ctx.long {
        map.insert(long, 0);
    }
}

impl Arguments<Empty, ()> {
    #[inline]
    pub fn new() -> Self {
        Self::new_with_sink(())
    }
}
impl<S> Arguments<Empty, S> {
    #[inline]
    pub fn new_with_sink(sink: S) -> Self {
        Self {
            args: Empty,
            sink,
            program_name: None,
            short_lut: empty_lut(),
            long_map: FxHashMap::default(),
        }
    }
    #[inline]
    pub fn add<'s, T: ArgumentValue<'s>>(self, argument: Arg<'s, T>) -> Arguments<Arg<'s, T>, S> {
        let Self {
            args: _,
            sink,
            program_name,
            mut short_lut,
            mut long_map,
        } = self;
        add_to_lut(&mut short_lut, &argument.ctx);
        add_to_map(&mut long_map, &argument.ctx);
        Arguments {
            args: argument,
            sink,
            program_name,
            short_lut,
            long_map,
        }
    }
}

impl<'s, T: ArgumentValue<'s>, S> Arguments<Arg<'s, T>, S> {
    #[inline]
    pub fn add<O: ArgumentValue<'s>>(
        self,
        argument: Arg<'s, O>,
    ) -> Arguments<More<'s, O, Arg<'s, T>>, S> {
        let Self {
            args,
            sink,
            program_name,
            mut short_lut,
            mut long_map,
        } = self;
        add_to_lut(&mut short_lut, &argument.ctx);
        add_to_map(&mut long_map, &argument.ctx);
        Arguments {
            args: More {
                rest: args,
                arg: argument,
            },
            sink,
            program_name,
            short_lut,
            long_map,
        }
    }
}

impl<'s, T: ArgumentValue<'s>, A: ArgumentList<'s>, S> Arguments<More<'s, T, A>, S> {
    #[inline]
    pub fn add<O: ArgumentValue<'s>>(
        self,
        argument: Arg<'s, O>,
    ) -> Arguments<More<'s, O, More<'s, T, A>>, S> {
        let Self {
            args,
            sink,
            program_name,
            mut short_lut,
            mut long_map,
        } = self;
        let len = args.len();
        assert!(len < u16::MAX as usize);
        add_to_lut(&mut short_lut, &argument.ctx);
        add_to_map(&mut long_map, &argument.ctx);
        Arguments {
            args: More {
                rest: args,
                arg: argument,
            },
            sink,
            program_name,
            short_lut,
            long_map,
        }
    }
}

impl Default for Arguments<Empty, ()> {
    #[inline]
    fn default() -> Self {
        Self::new_with_sink(())
    }
}

pub trait ArgumentSink<'s> {
    fn consume_value(&mut self, value: &'s str) -> Result<(), ArgError<'s>>;
}

impl<'s> ArgumentSink<'s> for () {
    #[inline(always)]
    fn consume_value(&mut self, _value: &'s str) -> Result<(), ArgError<'s>> {
        Ok(())
    }
}

impl<'s> ArgumentSink<'s> for Vec<&'s str> {
    #[inline(always)]
    fn consume_value(&mut self, value: &'s str) -> Result<(), ArgError<'s>> {
        self.push(value);
        Ok(())
    }
}

impl<'s, C: FnMut(&'s str) -> Result<(), ArgError<'s>>> ArgumentSink<'s> for C {
    #[inline(always)]
    fn consume_value(&mut self, value: &'s str) -> Result<(), ArgError<'s>> {
        self(value)
    }
}

impl<'s, A: ArgumentList<'s>, S: ArgumentSink<'s>> Arguments<A, S> {
    #[inline(always)]
    pub fn with_sink<NS: ArgumentSink<'s>>(self, new_sink: NS) -> Arguments<A, NS> {
        let Self {
            args,
            sink: _sink,
            program_name,
            short_lut,
            long_map,
        } = self;
        Arguments {
            args,
            sink: new_sink,
            program_name,
            short_lut,
            long_map,
        }
    }
    #[inline(always)]
    pub fn with_program_name(mut self, name: &'static str) -> Self {
        self.program_name = Some(name);
        self
    }
    #[inline(always)]
    pub fn parse(&mut self, args: &[&'s str]) -> Result<(), ArgError<'s>> {
        let mut source = ArgSource::new(args);
        while let Some(segment) = source.next() {
            match segment {
                ArgSegment::Short(0) => {
                    return Err(ArgError::UnknownShortOption('\0'));
                }
                ArgSegment::Short(short) => {
                    let idx = self.short_lut[(short - 1) as usize];
                    if idx == u16::MAX {
                        return Err(ArgError::UnknownShortOption(short as char));
                    }
                    self.args.capture_by_index(&mut source, idx)?;
                }
                ArgSegment::Long(long) => {
                    let Some(&idx) = self.long_map.get(long) else {
                        return Err(ArgError::UnknownLongOption(long));
                    };
                    self.args.capture_by_index(&mut source, idx)?;
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
            long_map: _,
        } = self;
        (args.into_values(), sink)
    }
    pub fn help_msg<'a>(&'a self) -> HelpMessage<'a, A, S> {
        HelpMessage(self)
    }
}
