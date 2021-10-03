use crate::assert::{Execution, Instance, MatcherTrait};
use std::borrow::Borrow;

impl<A> Instance<&[A]>
where
    A: PartialEq,
{
    pub fn contains<E>(&mut self, expected: E)
    where
        E: Borrow<A>,
    {
        let ok = matches!(self.actual.iter().find(|a| &expected.borrow() == a), Some(_));
        self.handle_execution(Execution {
            ok,
            log: "assertion failed: `(expectation ∈ actual)`".to_string(),
            nlog: "assertion failed: `(expectation ∉ actual)`".to_string(),
        });
    }

    pub fn eq_each<E>(&mut self, expected: &[E])
    where
        E: Borrow<A>,
    {
        if self.instance_config.negation {
            self.handle_execution(Execution {
                ok: true,
                log: "".to_string(),
                nlog: "eq_each assertion cannot be negated".to_string(),
            });
            return;
        }
        if self.actual.len() != expected.len() {
            let log = "expectation length is different from input length";
            self.handle_execution(Execution {
                ok: false,
                log: log.to_string(),
                nlog: "".to_string(),
            });
            return;
        }
        self.actual.iter().enumerate().for_each(|(pos, a)| {
            let ok = matches!(expected.get(pos), Some(e) if e.borrow() == a);
            self.handle_execution(Execution {
                ok,
                log: format!("assertion failed: `(expectation[{}] = actual[{}])`", pos, pos),
                nlog: "".to_string(),
            });
        });
    }
}

impl<A> Instance<&[A]> {
    pub fn has_len(&mut self, expected: usize) {
        let a_len = self.actual.as_ref().len();
        self.handle_execution(Execution {
            ok: a_len == expected,
            log: format!(
                r#"assertion failed: `(actual.len() == expectation)`
     actual.len(): `{:?}`
expectation: `{:?}`"#,
                a_len, expected
            ),
            nlog: format!(
                r#"assertion failed: `(actual.len() != expectation)`
     actual.len(): `{:?}`
expectation: `{:?}`"#,
                a_len, expected
            ),
        });
    }

    pub fn each<M>(&mut self, matchers: &[M])
    where
        M: MatcherTrait<A>,
    {
        if self.instance_config.negation {
            self.handle_execution(Execution {
                ok: true,
                log: "".to_string(),
                nlog: "each assertion cannot be negated".to_string(),
            });
            return;
        }
        if self.actual.len() != matchers.len() {
            let log = "matchers length is different from input length";
            self.handle_execution(Execution {
                ok: false,
                log: log.to_string(),
                nlog: "".to_string(),
            });
            return;
        }
        self.actual.iter().enumerate().for_each(|(pos, a)| {
            if let Some(matcher) = matchers.get(pos) {
                let ok = matcher.matcher_fn(a);
                self.handle_execution(Execution {
                    ok,
                    log: format!("{} - at position {}", matcher.log_fn(a), pos),
                    nlog: "".to_string(),
                });
            } else {
                self.handle_execution(Execution {
                    ok: false,
                    log: format!("matcher not found as position {}", pos),
                    nlog: "".to_string(),
                });
            }
        })
    }

    pub fn all<M>(&mut self, matcher: M)
    where
        M: MatcherTrait<A>,
    {
        if !self.instance_config.negation {
            self.actual.iter().enumerate().for_each(|(pos, a)| {
                let ok = matcher.matcher_fn(a);
                self.handle_execution(Execution {
                    ok,
                    log: format!("{} - at position {}", matcher.log_fn(a), pos),
                    nlog: "".to_string(),
                });
            })
        } else {
            let found = self.actual.iter().any(|a| !matcher.matcher_fn(a));
            if !found {
                self.handle_execution(Execution {
                    ok: true,
                    log: "".to_string(),
                    nlog: "assertion failed: `(matcher succeed for every item)`".to_string(),
                });
            }
        }
    }

    pub fn any<M>(&mut self, matcher: M)
    where
        M: MatcherTrait<A>,
    {
        if !self.instance_config.negation {
            let found = self.actual.iter().any(|a| matcher.matcher_fn(a));
            if !found {
                self.handle_execution(Execution {
                    ok: false,
                    log: "assertion failed: `(matcher failed for every item)`".to_string(),
                    nlog: "".to_string(),
                });
            }
        } else {
            let found = self.actual.iter().position(|a| matcher.matcher_fn(a));
            if let Some(a) = found {
                self.handle_execution(Execution {
                    ok: true,
                    log: "".to_string(),
                    nlog: format!("assertion failed: `(matcher succeed for item position {:?})`", a),
                });
            }
        }
    }
}
