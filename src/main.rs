mod audio;
mod logger;
mod ui;
mod config;
mod ext;
mod sub;

use std::str::FromStr;

use clap::Parser;

/// A TUI Subsonic music player
#[derive(Parser)]
#[clap(version, author)]
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

	if let Some(cache_path) = cfg.cache.location.as_ref() {
		if let Err(e) = std::fs::create_dir_all(cache_path) {
			eprintln!("[!] could not create cache dir: {e}");
			return;
		}

		let _ = sub::cache::DATA_CACHE_PATH.set(cache_path.clone());
	}

	let rt = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.expect("could not build tokio runtime");

	let player = audio::sink::AudioPlayer::new();

	let (provider, worker) = sub::Provider::create(cfg, player);

	let p = provider.clone();
	std::thread::spawn(move || {
		rt.block_on(async move {
			tokio::spawn(async move { worker.work().await; });
			match sub::mpris::serve(p).await {
				Ok(()) => std::future::pending().await,
				Err(e) => log::error!("error serving over MPRIS: {e}"),
			}
		})
	});

	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |panic_info| {
		ratatui::restore();
		hook(panic_info);
	}));

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
