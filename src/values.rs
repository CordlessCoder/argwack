use crate::{ArgError, ArgumentValue, arg::ArgContext, source::ArgSource};

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
