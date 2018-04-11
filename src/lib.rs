#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate byteorder;
extern crate bincode;

mod stuff;

pub use stuff::{
	Message,
	Stream,
	Endpoint,
	RecvError,
};

#[cfg(test)]
mod tests;