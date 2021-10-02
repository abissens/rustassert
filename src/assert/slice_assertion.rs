use crate::assert::{Execution, Instance, MatcherTrait};
use std::borrow::Borrow;
use std::fmt::Debug;

impl<A> Instance<&[A]>
where
    A: Debug + PartialEq,
{
    pub fn contains<E>(&mut self, expected: E)
    where
        E: Borrow<A> + Debug,
    {
        let ok = matches!(self.actual.iter().find(|a| &expected.borrow() == a), Some(_));
        self.handle_execution(Execution {
            ok,
            log: format!(
                r#"assertion failed: `(expectation ∈ actual)`
expectation: `{:?}`"#,
                expected
            ),
            nlog: format!(
                r#"assertion failed: `(expectation ∉ actual)`
expectation: `{:?}`"#,
                expected
            ),
        });
    }

    pub fn all<M>(&mut self, matcher: M)
    where
        M: MatcherTrait<A>,
    {
        if !self.instance_config.negation {
            self.actual.iter().for_each(|a| {
                let ok = matcher.matcher_fn(a);
                let log = matcher.log_fn(a);
                let nlog = matcher.nlog_fn(a);
                self.handle_execution(Execution { ok, log, nlog });
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
            let found = self.actual.iter().find(|a| matcher.matcher_fn(a));
            if let Some(a) = found {
                self.handle_execution(Execution {
                    ok: true,
                    log: "".to_string(),
                    nlog: format!("assertion failed: `(matcher succeed for item {:?})`", a),
                });
            }
        }
    }
}
