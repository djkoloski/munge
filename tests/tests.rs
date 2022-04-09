extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();

    config.mode = mode.parse().expect("Invalid mode");
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.target_rustcflags = Some("-L target/debug".to_string());
    config.clean_rmeta();
    // Uncomment to bless tests
    // config.bless = true;

    compiletest::run_tests(&config);
}

#[test]
fn compiletest() {
    run_mode("ui");
}
