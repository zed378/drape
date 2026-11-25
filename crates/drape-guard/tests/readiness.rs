#[test]
fn baseline_guard_cannot_acknowledge_readiness() {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_drape-guard"))
        .output()
        .expect("baseline Guard starts");
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("no readiness acknowledged"));
}
