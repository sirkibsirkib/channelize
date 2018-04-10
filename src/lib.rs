#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate byteorder;
extern crate bincode;

mod stuff;

pub use stuff::{
	Message,
	Endpoint,
	EE,
};

#[cfg(test)]
mod tests;