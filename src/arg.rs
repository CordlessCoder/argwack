use std::{fmt::Display, marker::PhantomData, num::NonZeroU8};
use crate::ArgumentValue;

#[derive(Debug, Default)]
pub struct Arg<'s, T: ArgumentValue<'s>> {
    pub(crate) ctx: ArgContext,
    pub(crate) out: T,
    pub(crate) _phantom: PhantomData<&'s T>,
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
