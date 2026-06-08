fn main() {
    argwack::test_helper(
        "benchmark-shorthand-same",
        &[
            "-q", "-w", "-e", "-u0", "-i1", "-o2", "-d0.0", "-f1.0", "-g2.0", "-lstr0", "-zstr1",
            "-xstr2",
        ],
    )
    .unwrap();
}
