extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let config = compiletest::Config {
        mode: mode.parse().expect("Invalid mode"),
        src_base: PathBuf::from(format!("tests/{}", mode)),
        target_rustcflags: Some("-L ../target/debug".to_string()),
        ..Default::default()
    };

    config.clean_rmeta();
    // Uncomment to bless tests
    // config.bless = true;

    compiletest::run_tests(&config);
}

#[test]
fn compiletest() {
    run_mode("ui");
}
