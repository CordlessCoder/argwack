use argwack::prelude::*;
use std::{hint::black_box, time::Instant};

fn main() {
    let mut args = Arguments::new()
        // Bunch-a bools
        .add(Arg::new_flag().with_short(b'q'))
        .add(Arg::new_flag().with_short(b'w'))
        .add(Arg::new_flag().with_short(b'e'))
        .add(Arg::new_flag().with_long("r"))
        .add(Arg::new_flag().with_long("t"))
        .add(Arg::new_flag().with_long("y"))
        // LONG, LONG MAAAAAAAAAAN
        .add(new_opt::<i64>().with_short(b'u'))
        .add(new_opt::<i64>().with_short(b'i'))
        .add(new_opt::<i64>().with_short(b'o'))
        .add(new_opt::<i64>().with_long("p"))
        .add(new_opt::<i64>().with_long("a"))
        .add(new_opt::<i64>().with_long("s"));
    let start = Instant::now();
    for _ in 0..1_000_000 {
        args.parse(black_box(&["", "-q", "-q", "-q", "-q", "-q"]))
            .unwrap();
        black_box(&args);
    }
    let took = start.elapsed();
    println!("1 million parses of 6 args each took {took:?}");
    // .add(Arg {
    //     short: Some(b'b'.try_into().unwrap()),
    //     out: None::<u32>,
    //     ..Default::default()
    // })
    // .add(Arg {
    //     short: Some(b'c'.try_into().unwrap()),
    //     out: None::<u32>,
    //     ..Default::default()
    // })
    // .add(Arg {
    //     short: Some(b'c'.try_into().unwrap()),
    //     out: None::<&str>,
    //     ..Default::default()
    // });
}
