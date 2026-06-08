fn main() {
    argwack::test_helper(
        "benchmark-mixed",
        &[
            "-q", "-w", "-e", "--r", "--t", "--y", "-u0", "-i1", "-o2", "--p=3", "--a=4", "--s=5",
            "-d", "0.0", "-f", "1.0", "-g", "2.0", "--h", "3.0", "--j", "4.0", "--k", "5.0",
            "-lstr0", "-zstr1", "-xstr2", "--c=str3", "--v=str4", "--b=str5",
        ],
    )
    .unwrap();
}
