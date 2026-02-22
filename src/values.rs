use std::str::FromStr;

use crate::{ArgError, ArgumentValue, arg::ArgContext, source::ArgSource};

impl<'s> ArgumentValue<'s> for bool {
    fn capture(
        &mut self,
        _ctx: &ArgContext,
        _args: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>> {
        *self = true;
        Ok(())
    }
}
impl<'s> ArgumentValue<'s> for u32 {
    fn capture(
        &mut self,
        _ctx: &ArgContext,
        _args: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>> {
        *self += 1;
        Ok(())
    }
}
impl<'s> ArgumentValue<'s> for Option<&'s str> {
    fn capture(
        &mut self,
        ctx: &ArgContext,
        args: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>> {
        let value = args
            .next_value()
            .ok_or(ArgError::MissingValueForOpt(*ctx))?;
        *self = Some(value);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OptFromStrWrapper<T: FromStr> {
    NotFound,
    Found(T),
}

impl<'s, T: FromStr> ArgumentValue<'s> for OptFromStrWrapper<T> {
    fn capture(
        &mut self,
        ctx: &ArgContext,
        args: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>> {
        let value = args
            .next_value()
            .ok_or(ArgError::MissingValueForOpt(*ctx))?;
        let parsed = value
            .parse()
            .ok()
            .ok_or(ArgError::InvalidValueForOpt(*ctx, value))?;
        *self = OptFromStrWrapper::Found(parsed);
        Ok(())
    }
}

#[derive(Debug)]
pub struct SetViaRef<'m, T>(pub &'m mut T);

impl<'m, 's, T: ArgumentValue<'s>> ArgumentValue<'s> for SetViaRef<'m, T> {
    fn capture(
        &mut self,
        ctx: &ArgContext,
        args: &mut ArgSource<'_, 's>,
    ) -> Result<(), ArgError<'s>> {
        self.0.capture(ctx, args)
    }
}
