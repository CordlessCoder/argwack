use argwack::prelude::*;

fn main() {
    let [mut q, mut w, mut e, mut r, mut t, mut y] = [false; _];
    let [mut u, mut i, mut o, mut p, mut a, mut s] = [None::<i64>; _];
    let [mut d, mut f, mut g, mut h, mut j, mut k] = [None::<f64>; _];
    let [mut l, mut z, mut x, mut c, mut v, mut b] = [None::<&str>; _];
    let mut name = None::<&str>;
    let args: Vec<String> = std::env::args().collect();
    let mut delimited = |_ctx, arg| {
        println!("{arg}");
        Ok(())
    };
    let mut opts = Arguments::new_with_positional(vec![ArgOut::from(&mut name)]);
    opts
        // Bools
        .add(Arg::new(&mut q).with_short(b'q'))
        .add(Arg::new(&mut w).with_short(b'w'))
        .add(Arg::new(&mut e).with_short(b'e'))
        .add(Arg::new(&mut r).with_long("r"))
        .add(Arg::new(&mut t).with_long("t"))
        .add(Arg::new(&mut y).with_long("y"))
        // Large ints
        .add(Arg::new(&mut u).with_short(b'u'))
        .add(Arg::new(&mut i).with_short(b'i'))
        .add(Arg::new(&mut o).with_short(b'o'))
        .add(Arg::new(&mut p).with_long("p"))
        .add(Arg::new(&mut a).with_long("a"))
        .add(Arg::new(&mut s).with_long("s"))
        // Lots of floats
        .add(Arg::new(&mut d).with_short(b'd'))
        .add(Arg::new(&mut f).with_short(b'f'))
        .add(Arg::new(&mut g).with_short(b'g'))
        .add(Arg::new(&mut h).with_long("h"))
        .add(Arg::new(&mut j).with_long("j"))
        .add(Arg::new(&mut k).with_long("k"))
        // Stringy cheese
        .add(Arg::new(&mut l).with_short(b'l'))
        .add(Arg::new(&mut z).with_short(b'z'))
        .add(Arg::new(&mut x).with_short(b'x'))
        .add(Arg::new(&mut c).with_long("c"))
        .add(Arg::new(&mut v).with_long("v"))
        .add(Arg::new(&mut b).with_long("b"))
        .add(Arg::delimited(',', &mut delimited).with_long("del"));

    opts.parse(&mut ArgSource::new(args.iter().skip(1).map(|s| s.as_str())))
        .unwrap();
}
