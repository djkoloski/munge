extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str, bless: bool) {
    let mut config = compiletest::Config {
        mode: mode.parse().expect("Invalid mode"),
        src_base: PathBuf::from(format!("tests/{}", mode)),
        target_rustcflags: Some("-L ../target/debug".to_string()),
        ..Default::default()
    };

    config.clean_rmeta();
    config.bless = bless;

    compiletest::run_tests(&config);
}

#[test]
fn compiletest() {
    // Set to `true` to bless tests
    let bless = true;

    run_mode("ui", bless);
}
