use backtrace::BacktraceFrame;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::panic::panic_any;
use std::rc::Rc;
use std::sync::Once;
use std::thread;

pub struct FailResult {
    pub log: String,
    pub bt: backtrace::Backtrace,
}

struct AssertionRef {
    failures: Vec<Box<dyn Fn()>>,
    f_handler: &'static dyn Fn(FailResult) -> Box<dyn Fn()>,
}

pub struct Assertion {
    rca: Rc<RefCell<AssertionRef>>,
}

fn default_f_handler(fr: FailResult) -> Box<dyn Fn()> {
    Box::new(move || {
        eprintln!("{}", fr.log);
        if !fr.bt.frames().is_empty() {
            eprintln!("{:?}", fr.bt);
        }
    })
}

pub fn new() -> Assertion {
    new_with_handler(&default_f_handler)
}

pub fn new_with_handler(handler: &'static dyn Fn(FailResult) -> Box<dyn Fn()>) -> Assertion {
    let rca = Rc::new(RefCell::new(AssertionRef { failures: vec![], f_handler: handler }));
    Assertion { rca }
}

pub fn that<A>(actual: A) -> Instance<A> {
    new().that(actual)
}

static MODULE_PATH: &[u8] = module_path!().as_bytes();

impl Assertion {
    pub fn that<A>(&mut self, actual: A) -> Instance<A> {
        Instance {
            parent: Rc::clone(&self.rca),
            actual: Box::new(actual),
            instance_config: InstanceConfig {
                negation: false,
                panic_immediately: false,
                backtrace: false,
            },
        }
    }

    fn current_path() -> Vec<u8> {
        MODULE_PATH.iter().chain("::".as_bytes().iter()).cloned().collect()
    }

    fn frame_in_assertion_module(frame: &BacktraceFrame) -> bool {
        let cp = Assertion::current_path();
        for s in frame.symbols() {
            if let Some(name) = s.name() {
                if name.as_bytes().starts_with(cp.as_slice()) {
                    return true;
                }
            }
        }
        false
    }

    fn backtrace_ignoring_current_mod(bt: backtrace::Backtrace) -> Vec<BacktraceFrame> {
        return bt
            .frames()
            .iter()
            .filter(|f| {
                for s in f.symbols() {
                    if let Some(name) = s.name() {
                        if name.as_bytes().starts_with(Assertion::current_path().as_slice()) {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();
    }

    fn failed_line_bt(bt: backtrace::Backtrace) -> Vec<BacktraceFrame> {
        let mut index_found = false;
        let mut last_f_index = 0;
        let f = bt.frames();
        for (i, f) in f.iter().enumerate() {
            if Assertion::frame_in_assertion_module(&f) {
                index_found = true;
                last_f_index = i;
            }
        }

        if !index_found || last_f_index + 1 == f.len() {
            return vec![];
        }

        return vec![f[last_f_index + 1].clone()];
    }
}

impl AssertionRef {
    fn fail(&mut self, instance_config: &InstanceConfig, log: String) {
        if instance_config.panic_immediately {
            panic!("{}", log);
        }
        let bt: backtrace::Backtrace;

        if instance_config.backtrace {
            bt = backtrace::Backtrace::from(Assertion::backtrace_ignoring_current_mod(backtrace::Backtrace::new()));
        } else {
            bt = backtrace::Backtrace::from(Assertion::failed_line_bt(backtrace::Backtrace::new()));
        }

        let f_handler = self.f_handler;
        self.failures.push(f_handler(FailResult { log, bt }))
    }
}

pub struct IgnorePanic();

static INIT_TAKE_HOOK: Once = Once::new();

impl Drop for AssertionRef {
    fn drop(&mut self) {
        if self.failures.is_empty() {
            return;
        }

        for f in self.failures.iter() {
            f();
        }

        if !thread::panicking() {
            INIT_TAKE_HOOK.call_once(|| {
                let default_panic = std::panic::take_hook();
                std::panic::set_hook(Box::new(move |info| {
                    if info.payload().downcast_ref::<IgnorePanic>().is_some() {
                        return;
                    }
                    default_panic(info);
                }));
            });
            panic_any(IgnorePanic());
        }
    }
}

pub struct Instance<A: ?Sized> {
    parent: Rc<RefCell<AssertionRef>>,
    actual: Box<A>,
    instance_config: InstanceConfig,
}

struct InstanceConfig {
    negation: bool,
    panic_immediately: bool,
    backtrace: bool,
}

struct Execution {
    ok: bool,
    log: String,
    nlog: String,
}

impl<A> Instance<A>
where
    A: ?Sized,
{
    pub fn not(&mut self) -> &mut Self {
        self.instance_config.negation = true;
        self
    }

    pub fn or_panic(&mut self) -> &mut Self {
        self.instance_config.panic_immediately = true;
        self
    }

    pub fn with_backtrace(&mut self) -> &mut Self {
        self.instance_config.backtrace = true;
        self
    }

    fn handle_execution(&mut self, e: Execution) {
        if !e.ok && !self.instance_config.negation {
            self.parent.borrow_mut().fail(&self.instance_config, e.log);
        }

        if e.ok && self.instance_config.negation {
            self.parent.borrow_mut().fail(&self.instance_config, e.nlog);
        }
    }
}

impl<A> Instance<A>
where
    A: Debug + PartialEq + ?Sized,
{
    pub fn is_eq<E>(&mut self, expected: E)
    where
        E: Borrow<A> + Debug,
    {
        let a = self.actual.as_ref();
        let b = expected.borrow();
        let ok = a.eq(b);
        self.handle_execution(Execution {
            ok,
            log: format!(
                r#"assertion failed: `(actual == expectation)`
     actual: `{:?}`
expectation: `{:?}`"#,
                self.actual, expected
            ),
            nlog: format!(
                r#"assertion failed: `(actual != expectation)`
     actual: `{:?}`
expectation: `{:?}`"#,
                self.actual, expected
            ),
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
}

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
}
