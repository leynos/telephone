//! The Telephone binary: prints a greeting.

fn main() {
    #[expect(
        clippy::print_stdout,
        reason = "the binary's only output is this greeting on standard output"
    )]
    {
        println!("Hello from Telephone!");
    }
}
