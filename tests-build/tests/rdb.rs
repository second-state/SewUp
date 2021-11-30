#[cfg(feature = "rdb")]
#[test]
fn compile_fail_full() {
    let t = trybuild::TestCases::new();

    t.pass("tests/compile-pass/sized_string.rs");
    t.pass("tests/compile-pass/table.rs");

    drop(t);
}
