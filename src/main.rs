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

	let paused = crate::ext::atomic::Flag::new(false);

	let sink = audio::sink::AudioSink::init(paused.clone())
		.expect("could not create audio sink");

	let auth = submarine::auth::AuthBuilder::new(&cfg.auth.username, "v1.16.1")
		.client_name("subtui")
		.hashed(&cfg.auth.password);
	let client = submarine::Client::new(&cfg.server.base, auth);

	let (provider, worker) = sub::Provider::create(client, sink, paused);

	let p = provider.clone();
	std::thread::spawn(|| tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.expect("could not build tokio runtime")
		.block_on(async move {
			tokio::spawn(async move { worker.work().await; });
			match sub::mpris::serve(p).await {
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
