//! Automatyczny zestaw self-testów silnika oraz bot AI (`AutoTestRunner`, `AutoTestStage`, `TestRecord`).
//! Całkowicie wolny od atrybutów `#[cfg]`.

mod overlay;
mod runner;
#[cfg(test)]
mod tests;

pub use runner::{AutoTestRunner, AutoTestStage, TestRecord};
