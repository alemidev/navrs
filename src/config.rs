#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct Config {
	#[serde(default)]
	pub server: ServerConfig,

	#[serde(default)]
	pub auth: AuthConfig,

	#[serde(default)]
	pub player: PlayerConfig,

	#[serde(default)]
	pub cache: CacheConfig,

	// #[serde(default)]
	// pub transcoding: TranscodingConfig,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct ServerConfig {
	#[serde_inline_default("http://localhost:8080/".to_string())]
	pub base: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct AuthConfig {
	#[serde(default)]
	pub username: String,

	#[serde(default)]
	pub password: String,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
#[serde(rename_all = "kebab-case")]
pub struct PlayerConfig {
	#[serde_inline_default("subtui".to_string())]
	pub device: String,

	#[serde_inline_default(5)]
	pub preload: usize,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
#[serde(rename_all = "kebab-case")]
pub struct CacheConfig {
	#[serde(default)]
	pub location: Option<String>,
}

// #[serde_inline_default::serde_inline_default]
// #[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
// #[serde(rename_all = "kebab-case")]
// pub struct TranscodingConfig {
// 	#[serde(default)]
// 	pub format: String,
// }

impl Config {
	pub fn load(path: Option<&std::path::PathBuf>) -> Result<Self, ConfigError> {
		match path {
			None => Ok(Self::default()),
			Some(path) => {
				let raw = std::fs::read_to_string(path)?;
				Ok(toml::from_str(&raw)?)
			}
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
	#[error("error reading file: {0}")]
	IO(#[from] std::io::Error),

	#[error("invalid Toml: {0}")]
	Toml(#[from] toml::de::Error),
}
