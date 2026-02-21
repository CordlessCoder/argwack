#[derive(Debug, Clone)]
enum Saved<'a> {
    Empty,
    Value(&'a str),
    Shorts(&'a [u8]),
}

#[derive(Debug, Clone)]
pub struct ArgSource<'slice, 'a> {
    args: &'slice [&'a str],
    saved: Saved<'a>,
}

impl<'slice, 'a> ArgSource<'slice, 'a> {
    pub fn new(args: &'slice [&'a str]) -> Self {
        Self {
            args,
            saved: Saved::Empty,
        }
    }
}

pub enum ArgSegment<'s> {
    Short(u8),
    Long(&'s str),
    Value(&'s str),
}

impl<'s> ArgSource<'_, 's> {
    pub fn next_value(&mut self) -> Option<&'s str> {
        match self.saved {
            Saved::Value(val) => {
                self.saved = Saved::Empty;
                return Some(val);
            }
            Saved::Shorts([]) => {
                self.saved = Saved::Empty;
            }
            Saved::Shorts(_) => {
                return None;
            }
            Saved::Empty => (),
        }
        let (&first, rest) = self.args.split_first()?;
        if first.starts_with('-') {
            return None;
        }
        self.args = rest;
        Some(first)
    }
    pub fn next(&mut self) -> Option<ArgSegment<'s>> {
        match self.saved {
            Saved::Empty => (),
            Saved::Value(val) => {
                self.saved = Saved::Empty;
                return Some(ArgSegment::Value(val));
            }
            Saved::Shorts([]) => {
                self.saved = Saved::Empty;
            }
            Saved::Shorts([first, rest @ ..]) => {
                self.saved = Saved::Shorts(rest);
                return Some(ArgSegment::Short(*first));
            }
        }
        let (&first, rest) = self.args.split_first()?;
        self.args = rest;
        match first.as_bytes() {
            [b'-', b'-', name @ ..] => {
                let mut name = name;
                if let Some(eq) = memchr::memchr(b'=', name) {
                    self.saved = Saved::Value(&first[2 + eq + 1..]);
                    name = &name[..eq];
                }
                Some(ArgSegment::Long(unsafe {
                    core::str::from_utf8_unchecked(name)
                }))
            }
            [b'-', short_name] => Some(ArgSegment::Short(*short_name)),
            [b'-', short_name, b'=', val @ ..] => {
                self.saved = Saved::Value(unsafe { core::str::from_utf8_unchecked(val) });
                Some(ArgSegment::Short(*short_name))
            }
            [b'-', short_name, more_shorts @ ..] => {
                self.saved = Saved::Shorts(more_shorts);
                Some(ArgSegment::Short(*short_name))
            }
            _ => Some(ArgSegment::Value(first)),
        }
    }
}

