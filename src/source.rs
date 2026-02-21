#[derive(Debug, Clone)]
enum Saved<'a> {
    Empty,
    Value(&'a str),
    Shorts(&'a [u8]),
}

#[derive(Debug, Clone)]
pub struct ArgSource<'a, I> {
    args: I,
    saved: Saved<'a>,
}

impl<'a, I: Iterator<Item = &'a str>> ArgSource<'a, I> {
    pub fn new(args: I) -> Self {
        Self {
            args,
            saved: Saved::Empty,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArgSegment<'s> {
    Short(u8),
    Long(&'s str),
    Value(&'s str),
}

impl<'s, I: Iterator<Item = &'s str>> ArgSource<'s, I> {
    pub fn next_value(&mut self) -> Option<&'s str> {
        match self.saved {
            Saved::Empty | Saved::Shorts([]) => (),
            Saved::Value(val) => {
                self.saved = Saved::Empty;
                return Some(val);
            }
            Saved::Shorts(looks_like_a_value) => {
                self.saved = Saved::Empty;
                return Some(core::str::from_utf8(looks_like_a_value).unwrap());
            }
        }
        let first = self.args.next()?;
        if first.starts_with('-') {
            return None;
        }
        Some(first)
    }
    pub fn next(&mut self) -> Option<ArgSegment<'s>> {
        match self.saved {
            Saved::Empty | Saved::Shorts([]) => (),
            Saved::Value(val) => {
                self.saved = Saved::Empty;
                return Some(ArgSegment::Value(val));
            }
            Saved::Shorts([first, rest @ ..]) => {
                self.saved = Saved::Shorts(rest);
                return Some(ArgSegment::Short(*first));
            }
        }
        let first = self.args.next()?;
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
