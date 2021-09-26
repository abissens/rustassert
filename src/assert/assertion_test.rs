#[cfg(test)]
mod tests {
    use crate::assert;
    use crate::assert::FailResult;
    use std::path::PathBuf;
    use std::{env, panic};

    const BASE_FOLDER: &'static str = env!("CARGO_MANIFEST_DIR");

    macro_rules! assert_panic_ignored {
        ($result:expr) => {
            assert!(match $result {
                Ok(_) => false,
                Err(d) => match d.downcast::<assert::IgnorePanic>() {
                    Ok(_) => true,
                    Err(_) => false,
                },
            })
        };
    }

    #[test]
    fn assert_eq_should_pass() {
        let mut assert = assert::new();

        assert.that("a").is_eq("a");
        assert.that("a").not().is_eq("b");

        assert.that(String::from("a")).is_eq(String::from("a"));
        assert.that(String::from("a")).not().is_eq(String::from("b"));

        assert.that(13).is_eq(13);
        assert.that(13).not().is_eq(14);

        assert.that(13).is_eq(&13);
        assert.that(13).not().is_eq(&14);

        assert.that(&13).is_eq(&13);
        assert.that(&13).not().is_eq(&14);

        #[derive(Debug, PartialEq)]
        struct S {
            a: i8,
            b: String,
        }

        assert.that(S { a: 13, b: String::from("A") }).is_eq(S { a: 13, b: String::from("A") });
        assert.that(S { a: 13, b: String::from("A") }).not().is_eq(S { a: 14, b: String::from("A") });

        assert.that(S { a: 13, b: String::from("A") }).is_eq(&S { a: 13, b: String::from("A") });
        assert.that(S { a: 13, b: String::from("A") }).not().is_eq(&S { a: 14, b: String::from("A") });

        assert.that(&S { a: 13, b: String::from("A") }).is_eq(&S { a: 13, b: String::from("A") });
        assert.that(&S { a: 13, b: String::from("A") }).not().is_eq(&S { a: 14, b: String::from("A") });

        assert.that(S { a: 13, b: String::from("A") }).is_eq(Box::new(S { a: 13, b: String::from("A") }));
        assert.that(S { a: 13, b: String::from("A") }).not().is_eq(Box::new(S { a: 14, b: String::from("A") }));

        assert.that(vec![1, 2, 3]).is_eq(vec![1, 2, 3]);
        assert.that(vec![1, 2, 3]).not().is_eq(vec![1, 2, 4]);
    }

    #[test]
    fn assert_eq_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(
                        fr.log,
                        r#"assertion failed: `(actual == expectation)`
     actual: `"a"`
expectation: `"b"`"#
                    );
                    assert_eq!(
                        BacktraceSum::from(&fr.bt),
                        BacktraceSum {
                            f: vec![FrameSum {
                                v: vec![FrameSymSum {
                                    name: "rustassert::assert::assertion_test::tests::assert_eq_should_fail::{{closure}}".to_string(),
                                    line: 88,
                                    file: PathBuf::from(BASE_FOLDER).join(file!()),
                                }]
                            }]
                        }
                    );
                })
            });
            assert.that("a").is_eq("b");
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_eq_should_fail_with_backtrace() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(
                        fr.log,
                        r#"assertion failed: `(actual == expectation)`
     actual: `"a"`
expectation: `"b"`"#
                    );
                    let bs = BacktraceSum::from(&fr.bt);
                    assert!(bs.f.len() > 10);
                    assert!(bs.f.contains(&FrameSum {
                        v: vec![FrameSymSum {
                            name: "rustassert::assert::assertion_test::tests::assert_eq_should_fail_with_backtrace::{{closure}}".to_string(),
                            line: 115,
                            file: PathBuf::from(BASE_FOLDER).join(file!()),
                        }]
                    }));
                })
            });
            assert.that("a").with_backtrace().is_eq("b");
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_eq_should_fail_with_negation() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(
                        fr.log,
                        r#"assertion failed: `(actual != expectation)`
     actual: `"a"`
expectation: `"a"`"#
                    );
                    assert_eq!(
                        BacktraceSum::from(&fr.bt),
                        BacktraceSum {
                            f: vec![FrameSum {
                                v: vec![FrameSymSum {
                                    name: "rustassert::assert::assertion_test::tests::assert_eq_should_fail_with_negation::{{closure}}".to_string(),
                                    line: 145,
                                    file: PathBuf::from(BASE_FOLDER).join(file!()),
                                }]
                            }]
                        }
                    );
                })
            });
            assert.that("a").not().is_eq("a");
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_has_len_should_pass() {
        let mut assert = assert::new();
        let a: &[i8] = &[1, 2, 3];
        assert.that(a).has_len(3);
        assert.that(vec![1, 2, 3].as_slice()).has_len(3);
        assert.that(vec![1, 2, 3].as_slice()).not().has_len(1);
    }

    #[test]
    fn assert_has_len_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(
                        fr.log,
                        r#"assertion failed: `(actual.len() == expectation)`
     actual.len(): `3`
expectation: `4`"#
                    );
                })
            });
            assert.that(vec![1, 2, 3].as_slice()).has_len(4);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_has_len_should_fail_with_negation() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(
                        fr.log,
                        r#"assertion failed: `(actual.len() != expectation)`
     actual.len(): `3`
expectation: `3`"#
                    );
                })
            });
            assert.that(vec![1, 2, 3].as_slice()).not().has_len(3);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_contains_should_pass() {
        let mut assert = assert::new();
        let a: &[i8] = &[1, 2, 3];
        assert.that(a).contains(2);
        assert.that(vec![1, 2, 3].as_slice()).contains(1);
        assert.that(vec![1, 2, 3].as_slice()).not().contains(0);
    }

    #[test]
    fn assert_contains_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(
                        fr.log,
                        r#"assertion failed: `(expectation ∈ actual)`
expectation: `4`"#
                    );
                })
            });
            assert.that(vec![1, 2, 3].as_slice()).contains(4);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_contains_should_fail_with_negation() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(
                        fr.log,
                        r#"assertion failed: `(expectation ∉ actual)`
expectation: `2`"#
                    );
                })
            });
            assert.that(vec![1, 2, 3].as_slice()).not().contains(2);
        });
        assert_panic_ignored!(result)
    }

    #[derive(Debug, PartialEq)]
    struct BacktraceSum {
        f: Vec<FrameSum>,
    }

    #[derive(Debug, PartialEq)]
    struct FrameSum {
        v: Vec<FrameSymSum>,
    }

    #[derive(Debug, PartialEq)]
    struct FrameSymSum {
        name: String,
        line: u32,
        file: PathBuf,
    }

    impl From<&backtrace::Backtrace> for BacktraceSum {
        fn from(bt: &backtrace::Backtrace) -> Self {
            BacktraceSum {
                f: bt.frames().iter().map(|f| FrameSum::from(f)).collect(),
            }
        }
    }

    impl From<&backtrace::BacktraceFrame> for FrameSum {
        fn from(f: &backtrace::BacktraceFrame) -> Self {
            FrameSum {
                v: f.symbols()
                    .iter()
                    .map(|s| FrameSymSum {
                        name: s.name().map(|n| n.as_str().unwrap_or_default().to_string()).unwrap_or(String::new()),
                        line: s.lineno().unwrap_or(0),
                        file: s.filename().map(|f| f.to_path_buf()).unwrap_or(PathBuf::new()),
                    })
                    .collect(),
            }
        }
    }
}
