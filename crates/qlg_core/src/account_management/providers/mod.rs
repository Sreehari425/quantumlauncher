//! Authentication provider implementations

mod microsoft;
mod elyby;
mod littleskin;

pub use microsoft::MicrosoftProvider;
pub use elyby::ElyByProvider;
pub use littleskin::LittleSkinProvider;
