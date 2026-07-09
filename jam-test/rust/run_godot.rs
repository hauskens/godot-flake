// Source: https://github.com/bytemeadow/godot-bevy/blob/main/examples/run_godot.rs
//
// jam-test has no game to run, so the non-itest binary just launches the (empty)
// extension in the accompanying Godot project. The real entry point is `--features
// itest`, which runs the integration tests for the shared jam crates.
#[cfg(not(feature = "itest"))]
fn main() {
    let runner = cargo_godot_lib::GodotRunner::create(
        env!("CARGO_PKG_NAME"),
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../godot"),
    );
    if let Err(e) = runner.execute() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(feature = "itest")]
fn main() {
    let runner = cargo_godot_lib::GodotRunner::create(
        env!("CARGO_PKG_NAME"),
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../godot"),
    );

    // Run with `cargo run --features itest` to run integration tests
    let runner = runner.godot_cli_arguments(vec![
        "--headless",
        "--scene",
        "res://addons/godot-bevy/test/TestRunner.tscn",
        "--quit-after",
        "10000",
    ]);

    let execute_result = runner.execute();

    // The test runner writes its pass/fail exit code to a file before Godot shuts
    // down, so prefer that over Godot's process status. A green suite can still trip
    // an unclean engine teardown (leaked ObjectDB instances -> signal on exit), and
    // that must not turn passing tests red. Only fall back to the launch error when
    // no exit code was written (e.g. the extension/class never loaded).
    match godot_bevy_test::exit_code::read_and_cleanup_exit_code() {
        Some(code) => std::process::exit(code),
        None => {
            if let Err(e) = execute_result {
                eprintln!("{e}");
            }
            std::process::exit(1);
        }
    }
}
