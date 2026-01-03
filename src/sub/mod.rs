pub mod cache;
pub mod provider;
pub mod mpris;

// TODO split down this file!
pub use provider::Provider;

pub type Song = submarine::data::Child;
pub type Id = String;
