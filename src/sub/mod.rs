pub mod cache;
pub mod provider;
pub mod mpris;
pub mod loader;

// TODO split down this file!
pub use provider::Provider;
pub use loader::Loader;

pub type Id = String;
pub type Artist = submarine::data::ArtistId3;
pub type Album = submarine::data::Child;
pub type Song = submarine::data::Child;
