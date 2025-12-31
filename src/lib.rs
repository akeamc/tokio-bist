//! A Built-In Self Test runner for Tokio users whose tests are organized into
//! a dynamic tree structure.
//!
//! ```no_run
#![doc = include_str!("../examples/hello.rs")]
//! ```

#![warn(clippy::pedantic)]
#![warn(missing_docs)]

use futures_util::{FutureExt, future::BoxFuture};
use tokio::task::JoinSet;

use crate::scons::Scons;

mod scons;

/// The result of a successful test case execution.
#[must_use]
pub struct Success {
    /// Optional warning message.
    pub warning: Option<anyhow::Error>,
    /// Branch off into more sub-cases. Implies that the parent test case passed.
    pub branches: Vec<Box<dyn TestCase>>,
}

/// The runner keeps track of all tests.
pub struct Runner {
    js: JoinSet<(usize, anyhow::Result<Success>)>,
    scons: Scons,
    n_spawned: usize,
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner {
    /// Create a new runner with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            js: JoinSet::new(),
            scons: Scons::new(),
            n_spawned: 0,
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.n_spawned;
        self.n_spawned += 1;
        id
    }

    fn spawn(&mut self, parent: &str, case: Box<dyn TestCase>) {
        let name = if parent.is_empty() {
            case.name()
        } else {
            format!("{} > {}", parent, case.name())
        };

        let id = self.next_id();
        self.scons.insert(id, name.clone());

        self.js.spawn(case.run().map(move |res| (id, res)));
    }

    /// Run the suite, beginning with `entrypoint`.
    ///
    /// # Panics
    ///
    /// This function will panic if any of the tests panic.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the tests fail.
    pub async fn run(mut self, entrypoint: Box<dyn TestCase>) -> anyhow::Result<()> {
        self.spawn("", entrypoint);

        let mut errored = false;

        while let Some((id, res)) = self
            .js
            .join_next()
            .await
            .transpose()
            .expect("spawned thread panicked")
        {
            let name = self.scons.remove(id, &res);

            match res {
                Ok(success) => {
                    for case in success.branches {
                        self.spawn(&name, case);
                    }
                }
                Err(_err) => {
                    errored = true;
                }
            }
        }

        self.scons.finalize();

        if errored {
            Err(anyhow::anyhow!("One or more checks failed"))
        } else {
            eprintln!("All checks passed!");

            Ok(())
        }
    }
}

/// A node in the test tree. The easiest way to create one is with [`test_fn`].
pub trait TestCase: Send {
    /// The name of the test case.
    fn name(&self) -> String;

    /// Run the test case. You should not call this directly; use [`Runner::run`] instead.
    fn run(self: Box<Self>) -> BoxFuture<'static, anyhow::Result<Success>>;
}

/// Create a new test case from a function.
pub fn test_fn<F, Fut>(name: impl Into<String>, f: F) -> Box<dyn TestCase>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<Success>> + Send + 'static,
{
    struct TestFn<F> {
        name: String,
        f: F,
    }

    impl<F, Fut> TestCase for TestFn<F>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = anyhow::Result<Success>> + Send + 'static,
    {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn run(self: Box<Self>) -> BoxFuture<'static, anyhow::Result<Success>> {
            (self.f)().boxed()
        }
    }

    Box::new(TestFn {
        name: name.into(),
        f,
    })
}

#[cfg(test)]
mod tests {
    use crate::{Runner, test_fn};

    #[tokio::test]
    #[should_panic(expected = "panic inside spawned task")]
    async fn test_panic() {
        Runner::new()
            .run(test_fn("entry", || async {
                panic!("panic inside spawned task");
            }))
            .await
            .unwrap();
    }
}
