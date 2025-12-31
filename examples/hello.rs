use std::time::Duration;

use anyhow::anyhow;
use futures_util::future::BoxFuture;
use rand::{
    Rng,
    distr::{Distribution, Uniform},
};
use tokio::time::sleep;
use tokio_bist::{Runner, Success, TestCase};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Runner::new()
        .run(Box::new(RandomBrancher { depth: 0, pid: 0 }))
        .await
}

struct RandomBrancher {
    depth: u32,
    pid: u16,
}

impl TestCase for RandomBrancher {
    fn name(&self) -> String {
        format!("{}", self.pid)
    }

    fn run(self: Box<Self>) -> BoxFuture<'static, anyhow::Result<Success>> {
        let ret = if self.depth >= 3 {
            if rand::random() {
                Ok(Success {
                    warning: None,
                    branches: vec![],
                })
            } else {
                Ok(Success {
                    warning: Some(anyhow!("Random warning")),
                    branches: vec![],
                })
            }
        } else {
            let mut rng = rand::rng();
            let branch_count = Uniform::new(3u32.saturating_sub(self.depth), 7)
                .unwrap()
                .sample(&mut rng);
            let mut branches = Vec::new();
            for _ in 0..branch_count {
                branches.push(Box::new(RandomBrancher {
                    depth: self.depth + 1,
                    pid: rng.random(),
                }) as Box<dyn TestCase>);
            }
            Ok(Success {
                warning: None,
                branches,
            })
        };

        Box::pin(async move {
            random_sleep().await;
            ret
        })
    }
}

async fn random_sleep() {
    let duration = Uniform::new(Duration::from_millis(50), Duration::from_millis(750))
        .unwrap()
        .sample(&mut rand::rng());

    sleep(duration).await;
}
