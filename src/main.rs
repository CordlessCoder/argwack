use argwack::prelude::*;
use std::{hint::black_box, time::Instant};

fn main() {
    let [mut q, mut w, mut e, mut r, mut t, mut y] = [false; _];
    let [mut u, mut i, mut o, mut p, mut a, mut s] = [None::<i64>; _];
    let [mut d, mut f, mut g, mut h, mut j, mut k] = [None::<f64>; _];
    let [mut l, mut z, mut x, mut c, mut v, mut b] = [None::<&str>; _];
    let mut args: Arguments<'_, '_, ()> = Arguments::new();
    args
        // Bools
        .add(Arg::from_out(ArgOut::Flag(&mut q)).with_short(b'q'))
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
        .add(Arg::new(&mut b).with_long("b"));

    let start = Instant::now();
    for _ in 0..1_000_000 {
        args.parse(black_box(
            ["", "-q", "-q", "-q", "-q", "-q", "-q"].iter().copied(),
        ))
        .unwrap();
        black_box(&args);
    }
    let took = start.elapsed();
    println!("1 million parses of 6 short args each took {took:?}");

    let opts = [
        "", "-q", "-w", "-e", "--r", "--t", "--y", "-u0", "-i1", "-o2", "--p=3", "--a=4", "--s=5",
        "-d0.0", "-f1.0", "-g2.0", "--h=3.0", "--j=4.0", "--k=5.0", "-lstr0", "-zstr1", "-xstr2",
        "--c=str3", "--v=str4", "--b=str5",
    ]
    .as_slice();
    let start = Instant::now();
    for _ in 0..1_000_000 {
        args.parse(black_box(opts.iter().copied())).unwrap();
        black_box(&args);
    }
    let took = start.elapsed();
    println!(
        "1 million parses of {} long args each took {took:?}",
        opts.len()
    );
    assert_eq!(x, Some("str2"));
    assert_eq!(b, Some("str5"));
}
