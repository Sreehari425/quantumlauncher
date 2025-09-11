//! Authentication provider implementations

mod elyby;
mod littleskin;
mod microsoft;
mod offline;

pub use elyby::ElyByProvider;
pub use littleskin::LittleSkinProvider;
pub use microsoft::MicrosoftProvider;
pub use offline::OfflineProvider;
