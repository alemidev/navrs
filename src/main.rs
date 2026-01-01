mod audio;
mod music;
mod app;
mod logger;
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

	let provider = music::SubsonicProvider::connect(&cli.host, &cli.username, &cli.password).unwrap();
	let _sink = audio::AudioSink::init(provider.clone()).unwrap();

	let term = ratatui::init();
	let res = app::App::new(provider).run(term);
	ratatui::restore();

	res.unwrap();
}
