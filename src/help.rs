use core::fmt::Display;

use crate::Arguments;

pub struct HelpMessage<'a, S>(pub(crate) &'a Arguments<'a, 'a, S>);

impl<S> Display for HelpMessage<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let args = self.0;
        if let Some(name) = args.program_name {
            writeln!(f, "{name}")?;
        }
        for arg in &args.args {
            let ctx = arg.ctx;
            writeln!(f, "{ctx}")?;
        }
        Ok(())
    }
}
