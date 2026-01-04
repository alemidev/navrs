pub mod atomic;
pub mod err;
mod tabularize;

pub use tabularize::tabularize;

pub fn format_secs_duration(secs: i32) -> String {
	format!("{}:{:02}", secs / 60, secs % 60)
}
