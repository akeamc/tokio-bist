use futures_util::{FutureExt, future::BoxFuture};
use tokio::task::JoinSet;

use crate::scons::Scons;

mod scons;

pub struct Runner {
    js: JoinSet<(String, Result)>,
    scons: Scons,
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner {
    pub fn new() -> Self {
        Self {
            js: JoinSet::new(),
            scons: Scons::new(),
        }
    }

    fn spawn(&mut self, parent: String, case: Box<dyn TestCase>) {
        let name = if parent.is_empty() {
            case.name()
        } else {
            format!("{} > {}", parent, case.name())
        };

        self.scons.insert(name.clone());

        self.js.spawn(case.run().map(|res| (name, res)));
    }

    pub async fn run(mut self, entrypoint: Box<dyn TestCase>) -> anyhow::Result<()> {
        self.spawn(String::new(), entrypoint);

        let mut errored = false;

        while let Some((name, res)) = self
            .js
            .join_next()
            .await
            .transpose()
            .expect("spawned thread panicked")
        {
            self.scons.remove(&name, &res);

            match res {
                Result::Ok => {}
                Result::Warn(_warn) => {}
                Result::Err(_err) => {
                    errored = true;
                }
                Result::Branch(branch) => {
                    for case in branch {
                        self.spawn(name.clone(), case);
                    }
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

#[must_use]
pub enum Result {
    Ok,
    Warn(anyhow::Error),
    Err(anyhow::Error),
    Branch(Vec<Box<dyn TestCase>>),
}

pub trait TestCase: Send {
    fn name(&self) -> String;

    fn run(self: Box<Self>) -> BoxFuture<'static, Result>;
}
