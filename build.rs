// build.rs
fn main() {
    // Enable full backtraces for better debugging
    println!("cargo:rustc-env=RUST_BACKTRACE=full");

    // Enable the SanitizerCoverage instrumentation for edge coverage
    println!("cargo:rustc-flag=-Clink-arg=-fsanitize-coverage=trace-pc-guard");

    // Optimize for better fuzzing performance
    println!("cargo:rustc-flag=-Ccodegen-units=1");
    println!("cargo:rustc-flag=-Copt-level=3");
}