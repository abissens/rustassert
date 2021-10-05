#[cfg(test)]
mod tests {
    use crate::assert;
    use crate::assert::FailResult;
    use crate::assert::SimpleMatcher;
    use crate::assert_panic_ignored;
    use crate::fn_matcher;
    use std::panic;

    #[test]
    fn assert_should_pass_after_mapping() {
        let mut assert = assert::new();
        assert.that(vec![1, 2, 3]).map(|a| format!("{}", a)).eq_each(&[String::from("1"), String::from("2"), String::from("3")]);
        assert.that(vec![1, 2, 3]).map(|a| format!("{}", a)).has_len(3);
        assert.that(vec![1, 2, 3]).map(|a| format!("{}", a)).not().has_len(0);
    }

    #[test]
    fn assert_has_len_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![1, 2, 3]).has_len(3);
        assert.that(vec![1, 2, 3]).not().has_len(1);
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
            assert.that(vec![1, 2, 3]).has_len(4);
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
            assert.that(vec![1, 2, 3]).not().has_len(3);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_contains_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![1, 2, 3]).contains(1);
        assert.that(vec![1, 2, 3]).not().contains(0);
    }

    #[test]
    fn assert_contains_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(expectation ∈ actual)`");
                })
            });
            assert.that(vec![1, 2, 3]).contains(4);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_contains_should_fail_with_negation() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(expectation ∉ actual)`");
                })
            });
            assert.that(vec![1, 2, 3]).not().contains(2);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_eq_each_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![1, 2, 3]).eq_each(&[1, 2, 3]);
        assert.that(vec![1, 2, 3]).eq_each(&[&1, &2, &3]);
    }

    #[test]
    fn assert_eq_each_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(expectation[1] = actual[1])`");
                })
            });
            assert.that(vec![1, 0, 3]).eq_each(&[1, 2, 3]);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_eq_each_should_fail_when_different_length() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "expectation length is different from input length");
                })
            });
            assert.that(vec![1, 2]).eq_each(&[1, 2, 3]);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_eq_each_should_prevent_negation() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "eq_each assertion cannot be negated");
                })
            });
            assert.that(vec![1, 0, 3]).not().eq_each(&[1, 2, 3]);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_each_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![1, 2, 3]).each(&[fn_matcher!(&|p| *p > 0), fn_matcher!(&|p| *p % 2 == 0), fn_matcher!(&|p| *p == 3)]);
    }

    #[test]
    fn assert_each_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(matcher \"&(|p| *p % 2 == 1)\" failed)` - at position 1");
                })
            });
            assert.that(vec![1, 2, 3]).each(&[fn_matcher!(&|p| *p > 0), fn_matcher!(&|p| *p % 2 == 1), fn_matcher!(&|p| *p == 3)]);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_each_should_fail_when_different_length() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "matchers length is different from input length");
                })
            });
            assert.that(vec![1, 2]).each(&[fn_matcher!(&|p| *p > 0), fn_matcher!(&|p| *p % 2 == 0), fn_matcher!(&|p| *p == 3)]);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_each_should_prevent_negation() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "each assertion cannot be negated");
                })
            });
            assert
                .that(vec![1, 2])
                .not()
                .each(&[fn_matcher!(&|p| *p > 0), fn_matcher!(&|p| *p % 2 == 0), fn_matcher!(&|p| *p == 3)]);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_all_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![]).all(fn_matcher!(&|a: &i32| *a > 0));
        assert.that(vec![1, 2, 3]).all(fn_matcher!(&|a| *a > 0));
        assert.that(vec![1, -2, 3]).not().all(fn_matcher!(&|a| *a > 0));
        assert.that(vec![-1, -2, -3]).not().all(fn_matcher!(&|a| *a > 0));
    }

    #[test]
    fn assert_all_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(matcher \"&(|a| *a > 0)\" failed)` - at position 1");
                })
            });
            assert.that(vec![1, -2, 3]).all(fn_matcher!(&|a| *a > 0));
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_all_should_fail_with_negation() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(matcher succeed for every item)`");
                })
            });
            assert.that(vec![1, 2, 3]).not().all(fn_matcher!(&|a| *a > 0));
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_any_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![]).not().any(fn_matcher!(&|a: &i32| *a > 0));
        assert.that(vec![-1, 2, -3]).any(fn_matcher!(&|a| *a > 0));
        assert.that(vec![-1, -2, -3]).not().any(fn_matcher!(&|a| *a > 0));
        assert.that(vec![1, 2, 3]).any(fn_matcher!(&|a| *a > 0));
    }

    #[test]
    fn assert_any_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(matcher failed for every item)`");
                })
            });
            assert.that(vec![-1, -2, -3]).any(fn_matcher!(&|a| *a > 0));
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_any_should_fail_with_negation() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(matcher succeed for item position 1)`");
                })
            });
            assert.that(vec![-1, 2, -3]).not().any(fn_matcher!(&|a| *a > 0));
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_single_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![-1, 2, -3]).any(fn_matcher!(&|a| *a > 0));
    }
}
