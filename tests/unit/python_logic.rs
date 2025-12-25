use wovensnake::core::lock::Lockfile;
use wovensnake::core::python;

#[test]
fn test_python_version_parsing_logic() {
    // This is a bit hard to test without mocking the Command output,
    // but we can at least test the Lockfile integration.
    let lock = Lockfile::new("test-project", "0.1.0", "3.12");
    assert_eq!(lock.python_version, "3.12");
}

#[test]
fn test_system_python_detection() {
    let version = python::get_system_python_version();
    // We can't guarantee a version exists in CI/Local, but we can check the format if it does
    if let Some(v) = version {
        assert!(v.contains('.'));
        let parts: Vec<&str> = v.split('.').collect();
        assert!(parts.len() >= 2);
    }
}
