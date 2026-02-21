use argwack::prelude::*;
use std::{hint::black_box, time::Instant};

fn main() {
    let [mut q, mut w, mut e, mut r, mut t, mut y] = [false; _];
    let [mut u, mut i, mut o, mut p, mut a, mut s] = [OptFromStrWrapper::<i64>::NotFound; _];
    let [mut d, mut f, mut g, mut h, mut j, mut k] = [OptFromStrWrapper::<f64>::NotFound; _];
    let [mut l, mut z, mut x, mut c, mut v, mut b] = [None::<&str>; _];
    let mut args = Arguments::new()
        // Bools
        .add(opt_by_ref(&mut q).with_short(b'q'))
        .add(opt_by_ref(&mut w).with_short(b'w'))
        .add(opt_by_ref(&mut e).with_short(b'e'))
        .add(opt_by_ref(&mut r).with_long("r"))
        .add(opt_by_ref(&mut t).with_long("t"))
        .add(opt_by_ref(&mut y).with_long("y"))
        // Large ints
        .add(opt_by_ref(&mut u).with_short(b'u'))
        .add(opt_by_ref(&mut i).with_short(b'i'))
        .add(opt_by_ref(&mut o).with_short(b'o'))
        .add(opt_by_ref(&mut p).with_long("p"))
        .add(opt_by_ref(&mut a).with_long("a"))
        .add(opt_by_ref(&mut s).with_long("s"))
        // Lots of floats
        .add(opt_by_ref(&mut d).with_short(b'd'))
        .add(opt_by_ref(&mut f).with_short(b'f'))
        .add(opt_by_ref(&mut g).with_short(b'g'))
        .add(opt_by_ref(&mut h).with_long("h"))
        .add(opt_by_ref(&mut j).with_long("j"))
        .add(opt_by_ref(&mut k).with_long("k"))
        // Stringy cheese
        .add(opt_by_ref(&mut l).with_short(b'l'))
        .add(opt_by_ref(&mut z).with_short(b'z'))
        .add(opt_by_ref(&mut x).with_short(b'x'))
        .add(opt_by_ref(&mut c).with_long("c"))
        .add(opt_by_ref(&mut v).with_long("v"))
        .add(opt_by_ref(&mut b).with_long("b"));

    let start = Instant::now();
    for _ in 0..1_000_000 {
        args.parse(black_box(&["", "-q", "-q", "-q", "-q", "-q", "-q"]))
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
        args.parse(black_box(opts)).unwrap();
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
