use core::{
    fmt::{Debug, Display},
    num::NonZeroU8,
    str::FromStr,
};

use alloc::boxed::Box;
use soa_rs::Soars;

use crate::{ArgError, Arguments, SubcommandSink, source::AnyArgSource};

type DelimitedCallback<'out, 'ext> =
    &'out mut dyn FnMut(&ArgContext, &str) -> Result<(), ArgError<'ext>>;
type Callback<'out, 'ext> =
    &'out mut dyn FnMut(&ArgContext, &mut dyn AnyArgSource<'ext>) -> Result<(), ArgError<'ext>>;

pub enum ArgOut<'out, 'ext> {
    Int(&'out mut Option<i64>),
    Float(&'out mut Option<f64>),
    Flag(&'out mut bool),
    Count(&'out mut u32),
    Str(&'out mut Option<&'ext str>),
    DelimitedCall(char, DelimitedCallback<'out, 'ext>),
    Call(
        &'out mut dyn FnMut(&ArgContext, &mut dyn AnyArgSource<'ext>) -> Result<(), ArgError<'ext>>,
    ),
    Subcommand(Box<Arguments<'out, 'ext, SubcommandSink>>),
}

impl<'o, 'e> ArgOut<'o, 'e> {
    pub(crate) fn capture(
        &mut self,
        ctx: &ArgContext,
        source: &mut impl AnyArgSource<'e>,
    ) -> Result<(), ArgError<'e>> {
        use ArgOut::*;
        match self {
            Flag(f) => **f = true,
            Count(c) => **c += 1,
            Str(s) => {
                let value = source
                    .next_value()
                    .ok_or(ArgError::MissingValueForOpt(*ctx))?;
                **s = Some(value);
            }
            Int(i) => {
                **i = Some(capture_from_str(ctx, source)?);
            }
            Float(f) => {
                **f = Some(capture_from_str(ctx, source)?);
            }
            Call(c) => {
                c(ctx, source)?;
            }
            DelimitedCall(del, c) => {
                let val = source
                    .next_value()
                    .ok_or(ArgError::MissingValueForOpt(*ctx))?;
                for val in val.split(*del) {
                    c(ctx, val)?;
                }
            }
            Subcommand(sub) => match sub.parse(source) {
                Err(ArgError::UnexpectedPositionalOpt(val)) => {
                    source.inject_val(val);
                }
                otherwise => return otherwise,
            },
        }
        Ok(())
    }
}

impl Debug for ArgOut<'_, '_> {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use ArgOut::*;
        match self {
            Int(v) => write!(out, "Int({v:?})"),
            Float(v) => write!(out, "Float({v:?})"),
            Flag(v) => write!(out, "Flag({v})"),
            Count(v) => write!(out, "Count({v})"),
            Str(v) => write!(out, "Str({v:?})"),
            Call(_) => write!(out, "Call"),
            DelimitedCall(del, _) => write!(out, "DelimitedCall({del:?})"),
            Subcommand(_) => write!(out, "Subcommand"),
        }
    }
}

impl<'out> From<&'out mut Option<f64>> for ArgOut<'out, '_> {
    fn from(value: &'out mut Option<f64>) -> Self {
        ArgOut::Float(value)
    }
}
impl<'out> From<&'out mut Option<i64>> for ArgOut<'out, '_> {
    fn from(value: &'out mut Option<i64>) -> Self {
        ArgOut::Int(value)
    }
}
impl<'out, 'ext> From<&'out mut Option<&'ext str>> for ArgOut<'out, 'ext> {
    fn from(value: &'out mut Option<&'ext str>) -> Self {
        ArgOut::Str(value)
    }
}
impl<'out> From<&'out mut bool> for ArgOut<'out, '_> {
    fn from(value: &'out mut bool) -> Self {
        ArgOut::Flag(value)
    }
}

#[derive(Debug, Soars)]
#[must_use]
pub struct Arg<'out, 'ext> {
    pub out: ArgOut<'out, 'ext>,
    pub ctx: ArgContext,
}

pub fn capture_from_str<'ext, T: FromStr>(
    ctx: &ArgContext,
    source: &mut impl AnyArgSource<'ext>,
) -> Result<T, ArgError<'ext>> {
    let value = source
        .next_value()
        .ok_or(ArgError::MissingValueForOpt(*ctx))?;
    value
        .parse()
        .ok()
        .ok_or(ArgError::InvalidValueForOpt(*ctx, value))
}

impl<'o, 'e> Arg<'o, 'e> {
    #[inline(always)]
    pub fn new<T: Into<ArgOut<'o, 'e>>>(val: T) -> Self {
        Self::from_out(val.into())
    }
    #[inline(always)]
    pub fn callback(cb: Callback<'o, 'e>) -> Self {
        Self::from_out(ArgOut::Call(cb))
    }
    #[inline(always)]
    pub fn delimited(del: char, cb: DelimitedCallback<'o, 'e>) -> Self {
        Self::from_out(ArgOut::DelimitedCall(del, cb))
    }
    #[inline(always)]
    pub fn from_out(val: ArgOut<'o, 'e>) -> Self {
        Self {
            ctx: ArgContext::empty(),
            out: val,
        }
    }
    #[inline(always)]
    pub fn with_short(mut self, short: u8) -> Self {
        self.ctx.short = short.try_into().ok();
        self
    }
    #[inline(always)]
    pub fn with_long(mut self, long: &'static &'static str) -> Self {
        self.ctx.long = Some(long);
        self
    }
    #[inline(always)]
    pub fn with_help(mut self, help: &'static &'static str) -> Self {
        self.ctx.help = Some(help);
        self
    }
}

#[derive(Default, Clone, Copy)]
pub struct ArgContext {
    pub short: Option<NonZeroU8>,
    pub long: Option<&'static str>,
    pub help: Option<&'static str>,
}

impl Debug for ArgContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ArgContext")
            .field("short", &self.short.map(|v| v.get() as char))
            .field("long", &self.long)
            .field("help", &self.help)
            .finish()
    }
}

impl Display for ArgContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            short: None,
            long: None,
            help: None,
        }
    }
}
