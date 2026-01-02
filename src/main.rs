mod app;
mod audio;
mod logger;
mod music;
mod mpris;
mod ui;

use clap::Parser;

/// A TUI Subsonic music player
#[derive(Parser)]
struct Cli {
	/// username to use for authentication against server
	#[arg(short, long)]
	username: String,

	/// password to use for authentication against server
	#[arg(short, long)]
	password: String,

	/// address of your server
	#[arg(short = 'H', long)]
	host: String,
}

fn main() {
	logger::init().unwrap();
	let cli = Cli::parse();

	let provider =
		music::SubsonicProvider::connect(&cli.host, &cli.username, &cli.password).unwrap();
	let _sink = audio::AudioSink::init(provider.clone()).unwrap();

	let p = provider.clone();
	std::thread::spawn(|| tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.expect("could not build tokio runtime")
		.block_on(async move { 
			match mpris::serve(p).await {
				Ok(()) => std::future::pending().await,
				Err(e) => log::error!("error serving over MPRIS: {e}"),
			}
		})
	);

	libnotify::init("subtui");
	let term = ratatui::init();
	let res = app::App::new(provider).run(term);
	ratatui::restore();
	libnotify::uninit();

	res.unwrap();
}
