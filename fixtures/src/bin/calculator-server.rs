//! The Slim server behind `samples/calculator`, used to run real tests.

use rust_slim::{PortSlimServer, fixture};
use std::env::args;
use std::error::Error;
use std::process;
use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
pub struct Calculator {
    a: i64,
    b: i64,
}

#[fixture("Sample.Fixtures.Calculator")]
impl Calculator {
    #[slim(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_a(&mut self, a: i64) {
        self.a = a;
    }

    pub fn set_b(&mut self, b: i64) {
        self.b = b;
    }

    #[must_use]
    pub fn sum(&self) -> i64 {
        self.a + self.b
    }

    #[must_use]
    pub fn mul(&self) -> i64 {
        self.a * self.b
    }

    /// Divides `a` by `b`.
    ///
    /// # Errors
    ///
    /// Fails when dividing by zero, which temoc reports as an exception.
    pub fn div(&self) -> Result<i64, String> {
        self.a
            .checked_div(self.b)
            .ok_or_else(|| "division by zero".to_owned())
    }

    /// The sum, unless `a` is zero. Then it never answers, which leaves a run
    /// in progress for the spec to look at.
    #[must_use]
    pub fn hang_on_zero(&self) -> i64 {
        if self.a == 0 {
            loop {
                sleep(Duration::from_secs(1));
            }
        }
        self.sum()
    }

    /// The sum, unless `a` is zero. Then the server stops in the middle of a run.
    #[must_use]
    pub fn die_on_zero(&self) -> i64 {
        if self.a == 0 {
            process::exit(3);
        }
        self.sum()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = PortSlimServer::listen_from_args(args().skip(1))?;
    server.add_fixture::<Calculator>();
    server.run()?;
    Ok(())
}
