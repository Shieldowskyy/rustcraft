//! Shared engine error types and core domain event definitions.
//! Pure data contracts with no platform or rendering side effects.
//! Used across `core`, `platform`, and `features` boundaries.

use std::fmt;

use crate::core::coords::{ChunkPos, WorldPos};
use crate::core::i18n::Language;

/// Unified error type for engine, persistence, platform, and network boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineError {
    Io(String),
    Serialization(String),
    Platform(String),
    Graphics(String),
    Network(String),
    InvalidState(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Platform(msg) => write!(f, "Platform error: {msg}"),
            Self::Graphics(msg) => write!(f, "Graphics error: {msg}"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<std::io::Error> for EngineError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<EngineError> for String {
    fn from(err: EngineError) -> Self {
        err.to_string()
    }
}

/// High-level engine events emitted across subsystem boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreEvent {
    LanguageChanged(Language),
    SettingsChanged,
    ChunkDirty(ChunkPos),
    BlockModified(WorldPos),
    WorldSaved { slot: u8 },
}

