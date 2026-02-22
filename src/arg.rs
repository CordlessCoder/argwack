use crate::ArgumentValue;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    num::NonZeroU8,
};

#[derive(Debug, Default)]
pub struct Arg<'s, T: ArgumentValue<'s>> {
    pub ctx: ArgContext,
    pub out: T,
    pub(crate) _phantom: PhantomData<&'s ()>,
}

#[derive(Default, Clone, Copy)]
pub struct ArgContext {
    pub short: Option<NonZeroU8>,
    pub long: Option<&'static str>,
    pub help: Option<&'static str>,
}

impl Debug for ArgContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArgContext")
            .field("short", &self.short.map(|v| v.get() as char))
            .field("long", &self.long)
            .field("help", &self.help)
            .finish()
    }
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
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            short: None,
            long: None,
            help: None,
        }
    }
}
