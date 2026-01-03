use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

static LOGGER: SimpleLogger = SimpleLogger {
	buffer: std::sync::RwLock::new(vec![]),
};

struct SimpleLogger {
	buffer: std::sync::RwLock<Vec<String>>,
}

impl log::Log for SimpleLogger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= Level::Debug
	}

	fn log(&self, record: &Record) {
		if self.enabled(record.metadata()) {
			LOGGER
				.buffer
				.write()
				.unwrap()
				.push(format!("{} - {}", record.level(), record.args()));
		}
	}

	fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
	log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Debug))
}

pub fn all() -> Vec<String> {
	LOGGER.buffer.read().unwrap().clone()
}
