use std::process::ExitCode;

pub fn version() -> ExitCode {
    println!(
        "litho {} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA_SHORT"),
        env!("VERGEN_GIT_COMMIT_DATE")
    );

    ExitCode::SUCCESS
}
