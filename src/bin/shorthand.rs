fn main() {
    argwack::test_helper(
        "benchmark-shorthand",
        &[
            "-q", "-w", "-e", "-u", "0", "-i", "1", "-o", "2", "-d", "0.0", "-f", "1.0", "-g",
            "2.0", "-l", "str0", "-z", "str1", "-x", "str2",
        ],
    )
    .unwrap();
}
