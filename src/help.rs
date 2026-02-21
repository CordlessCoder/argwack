use std::fmt::Display;

use crate::{ArgumentList, Arguments};

pub struct HelpMessage<'a, A, S>(pub(crate) &'a Arguments<A, S>);

impl<'s, 'a, A: ArgumentList<'s>, S> Display for HelpMessage<'a, A, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self.0;
        if let Some(name) = args.program_name {
            writeln!(f, "{name}")?;
        }
        args.args.visit_ctxs(&mut |ctx| writeln!(f, "{ctx}"))
    }
}
