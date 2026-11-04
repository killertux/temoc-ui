//! The Slim server behind `samples/calculator`, used to run real tests.

use rust_slim::{PortSlimServer, fixture};
use std::env::args;
use std::error::Error;

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
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = PortSlimServer::listen_from_args(args().skip(1))?;
    server.add_fixture::<Calculator>();
    server.run()?;
    Ok(())
}
