mod audio;
mod logger;
mod music;
mod mpris;
mod ui;
mod config;
mod ext;

use std::str::FromStr;

use clap::Parser;

/// A TUI Subsonic music player
#[derive(Parser)]
struct Cli {
	/// path to a specific config file, otherwise searches .config/subtui/config.toml
	#[arg(short, long)]
	config: Option<std::path::PathBuf>,
}

fn main() {
	logger::init().unwrap();
	let cli = Cli::parse();

	let cfg = match config::Config::load(Some(&default_config_path(cli.config))) {
		Ok(c) => c,
		Err(e) => return println!("invalid config: {e}"),
	};

	// TODO ew but whatever i guess...
	let (provider, fut) =
		music::SubsonicProvider::create(cfg.server.base, cfg.auth.username, cfg.auth.password);
	let _sink = audio::AudioSink::init(provider.clone()).unwrap();

	let p = provider.clone();
	std::thread::spawn(|| tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.expect("could not build tokio runtime")
		.block_on(async move {
			tokio::spawn(fut);
			match mpris::serve(p).await {
				Ok(()) => std::future::pending().await,
				Err(e) => log::error!("error serving over MPRIS: {e}"),
			}
		})
	);

	let term = ratatui::init();
	let res = ui::App::new(provider).run(term);
	ratatui::restore();

	res.unwrap();
}

fn default_config_path(force: Option<std::path::PathBuf>) -> std::path::PathBuf {
	if let Some(ovr) = force {
		return ovr;
	}

	let home = std::env::var("HOME").unwrap_or("/root".to_string());

	let mut out = std::path::PathBuf::from_str(&home).unwrap(); // infallible
	out.push(".config");
	out.push("subtui");
	out.push("config.toml");
	out
}
