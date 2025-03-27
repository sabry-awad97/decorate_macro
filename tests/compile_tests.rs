use std::path::PathBuf;

#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();

    // Ensure the test directories exist
    let test_dirs = ["tests/pass", "tests/fail"];
    for dir in test_dirs.iter() {
        std::fs::create_dir_all(dir).unwrap();
    }

    // Run the tests
    t.compile_fail("tests/fail/*.rs");
    t.pass("tests/pass/*.rs");
}

#[test]
fn check_stderr_files() {
    let fail_dir = PathBuf::from("tests/fail");
    if !fail_dir.exists() {
        return;
    }

    // Check that each .rs file has a corresponding .stderr file
    for entry in std::fs::read_dir(fail_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let stderr_path = path.with_extension("stderr");
            assert!(stderr_path.exists(), "Missing .stderr file for {:?}", path);
        }
    }
}
