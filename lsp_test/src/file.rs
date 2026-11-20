mod toml;
mod unit;

use anyhow::Result;
use std::{fs, path::Path};

pub use toml::{Expect, ExpectSpan, Expectation};
pub use unit::TestUnit;

pub struct TestUnits {
    pub units: Vec<TestUnit>,
    pub items: usize,
}

impl TestUnits {
    /// Create `TestUnits` table for testing.
    pub fn new(parent: &Path) -> Result<Self> {
        let mut units = vec![];

        for dir in fs::read_dir(parent)? {
            let path = dir?.path();
            let unit = TestUnit::read_from_single_dir(path)?;
            units.push(unit);
        }

        let items = units.len();

        Ok(Self { units, items })
    }

    /// Bless JSON snapshots.
    pub fn bless(&mut self) -> Result<()> {
        for unit in &mut self.units {
            unit.bless()?;
        }
        Ok(())
    }

    /// Check all tests.
    pub fn check(&self) -> Result<()> {
        let mut fails = vec![];
        for unit in &self.units {
            let (unit, ok) = unit.check().map(|ok| (unit, ok))?;
            if !ok {
                fails.push(unit.name.as_str());
            }
        }

        anyhow::ensure!(
            fails.is_empty(),
            "{} of {} units failed: {}",
            fails.len(),
            self.items,
            fails.join(", ")
        );
        Ok(())
    }
}
