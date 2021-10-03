#[cfg(test)]
mod tests {
    use crate::assert;
    use crate::assert::FailResult;
    use crate::assert::SimpleMatcher;
    use crate::assert_panic_ignored;
    use crate::fn_matcher;
    use std::panic;

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
                    assert_eq!(fr.log, "assertion failed: `(expectation ∈ actual)`");
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
                    assert_eq!(fr.log, "assertion failed: `(expectation ∉ actual)`");
                })
            });
            assert.that(vec![1, 2, 3].as_slice()).not().contains(2);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_eq_each_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![1, 2, 3].as_slice()).eq_each(&[1, 2, 3]);
        assert.that(vec![1, 2, 3].as_slice()).eq_each(&[&1, &2, &3]);
    }

    #[test]
    fn assert_eq_each_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(expectation[1] = actual[1])`");
                })
            });
            assert.that(vec![1, 0, 3].as_slice()).eq_each(&[1, 2, 3]);
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
            assert.that(vec![1, 2].as_slice()).eq_each(&[1, 2, 3]);
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
            assert.that(vec![1, 0, 3].as_slice()).not().eq_each(&[1, 2, 3]);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_each_should_pass() {
        let mut assert = assert::new();
        assert
            .that(vec![1, 2, 3].as_slice())
            .each(&[fn_matcher!(&|p| *p > 0), fn_matcher!(&|p| *p % 2 == 0), fn_matcher!(&|p| *p == 3)]);
    }

    #[test]
    fn assert_each_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(matcher \"&(|p| *p % 2 == 1)\" failed)` - at position 1");
                })
            });
            assert
                .that(vec![1, 2, 3].as_slice())
                .each(&[fn_matcher!(&|p| *p > 0), fn_matcher!(&|p| *p % 2 == 1), fn_matcher!(&|p| *p == 3)]);
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
            assert
                .that(vec![1, 2].as_slice())
                .each(&[fn_matcher!(&|p| *p > 0), fn_matcher!(&|p| *p % 2 == 0), fn_matcher!(&|p| *p == 3)]);
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
                .that(vec![1, 2].as_slice())
                .not()
                .each(&[fn_matcher!(&|p| *p > 0), fn_matcher!(&|p| *p % 2 == 0), fn_matcher!(&|p| *p == 3)]);
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_all_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![].as_slice()).all(fn_matcher!(&|a: &i32| *a > 0));
        assert.that(vec![1, 2, 3].as_slice()).all(fn_matcher!(&|a| *a > 0));
        assert.that(vec![1, -2, 3].as_slice()).not().all(fn_matcher!(&|a| *a > 0));
        assert.that(vec![-1, -2, -3].as_slice()).not().all(fn_matcher!(&|a| *a > 0));
    }

    #[test]
    fn assert_all_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(matcher \"&(|a| *a > 0)\" failed)` - at position 1");
                })
            });
            assert.that(vec![1, -2, 3].as_slice()).all(fn_matcher!(&|a| *a > 0));
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
            assert.that(vec![1, 2, 3].as_slice()).not().all(fn_matcher!(&|a| *a > 0));
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_any_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![].as_slice()).not().any(fn_matcher!(&|a: &i32| *a > 0));
        assert.that(vec![-1, 2, -3].as_slice()).any(fn_matcher!(&|a| *a > 0));
        assert.that(vec![-1, -2, -3].as_slice()).not().any(fn_matcher!(&|a| *a > 0));
        assert.that(vec![1, 2, 3].as_slice()).any(fn_matcher!(&|a| *a > 0));
    }

    #[test]
    fn assert_any_should_fail() {
        let result = panic::catch_unwind(|| {
            let mut assert = assert::new_with_handler(&|fr: FailResult| {
                Box::new(move || {
                    assert_eq!(fr.log, "assertion failed: `(matcher failed for every item)`");
                })
            });
            assert.that(vec![-1, -2, -3].as_slice()).any(fn_matcher!(&|a| *a > 0));
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
            assert.that(vec![-1, 2, -3].as_slice()).not().any(fn_matcher!(&|a| *a > 0));
        });
        assert_panic_ignored!(result)
    }

    #[test]
    fn assert_single_should_pass() {
        let mut assert = assert::new();
        assert.that(vec![-1, 2, -3].as_slice()).any(fn_matcher!(&|a| *a > 0));
    }
}
