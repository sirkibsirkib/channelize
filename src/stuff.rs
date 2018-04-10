use serde::{
	Serialize,
	// Deserialize,
	de::DeserializeOwned,
};

use byteorder::{
	LittleEndian,
	ReadBytesExt,
	WriteBytesExt,
};

use ::bincode;

use ::std::{
	io,
	fmt::Debug,
	convert::From,
	marker::PhantomData,
	io::prelude::*,
	net::TcpStream,
};

const LEN_BYTES: usize = 4; 
pub trait Message: Serialize + DeserializeOwned + Sized + Debug {}

#[derive(Debug)]
pub struct Endpoint<M>
where M: Message {
	stream: TcpStream,
	buf: Vec<u8>,
	buf_occupancy: usize,
	payload_bytes: Option<u32>,
	phantom: PhantomData<M>,
}

#[derive(Debug)]
pub enum EE {
	NoBytes,
	InsufficientBytes(usize),
	SocketErr(io::Error)
}

impl From<io::Error> for EE {
	fn from(t: io::Error) -> EE {
		EE::SocketErr(t)
	}
}


impl<M> Endpoint<M>
where M: Message {
	pub fn new(stream: TcpStream) -> Endpoint<M> {
		stream.set_nonblocking(true)
		.expect("Failed to set given socket to non-blocking mode!");
		Endpoint {
			stream: stream,
			buf: vec![],
			buf_occupancy: 0,
			payload_bytes: None,
			phantom: PhantomData,
		}
	}

	fn ensure_buf_capacity(&mut self, capacity: usize) {
		while self.buf.len() < capacity {
			self.buf.push(0u8);
		}
		self.buf.shrink_to_fit(); // >??? GOOD OR NAH
	}

	pub fn send(&mut self, m: &M) -> Result<usize, EE> {
		let encoded: Vec<u8> = bincode::serialize(&m).expect("nawww");
		let len = encoded.len();
		if len > ::std::u32::MAX as usize {
			panic!("`send()` can only handle payloads up to std::u32::MAX");
		}
		let mut encoded_len = vec![];
		encoded_len.write_u32::<LittleEndian>(len as u32).expect("wahey");
		self.stream.write(&encoded_len)?;
		self.stream.write(&encoded)?;
		Ok(len)
	}

	pub fn new_change_type<Q: Message>(self) -> Endpoint<Q> {
		Endpoint {
			stream: self.stream,
			buf: self.buf,
			buf_occupancy: self.buf_occupancy,
			payload_bytes: self.payload_bytes,
			phantom: PhantomData,
		}
	}

	pub fn recv(&mut self) -> Result<M, EE> {
		self.stream.set_nonblocking(false)?;
		loop {
			match self.try_recv() {
				Err(EE::NoBytes) => (),
				Err(EE::InsufficientBytes(_)) => (),
				Ok(m) => {
					self.stream.set_nonblocking(true)?;
					return Ok(m)
				}
				Err(x) => return Err(x),
			}
		}
	}

	pub fn try_recv(&mut self) -> Result<M, EE> {
		if self.payload_bytes.is_none() {
			self.ensure_buf_capacity(LEN_BYTES);
			self.buf_occupancy +=
				self.stream.read(&mut self.buf[self.buf_occupancy..LEN_BYTES])?;
			if self.buf_occupancy == 4 {
				self.payload_bytes = Some(
					(&self.buf[0..LEN_BYTES]).read_u32::<LittleEndian>()
					.expect("naimen")
				);
			}
		}
		if let Some(pb) = self.payload_bytes {
			// try to get the payload bytes
			let buf_end: usize = LEN_BYTES + pb as usize;
			self.ensure_buf_capacity(buf_end);
			self.buf_occupancy +=
				self.stream.read(&mut self.buf[LEN_BYTES..buf_end])?;

			if self.buf_occupancy == buf_end {
				// read message to completion!
				let decoded: M = bincode::deserialize(
					&self.buf[LEN_BYTES..buf_end]
				).expect("jirre");
				self.buf_occupancy = 0;
				self.payload_bytes = None;
				return Ok(decoded);
			}
		}
		if self.buf_occupancy == 0 {
			Err(EE::NoBytes)
		} else {
			Err(EE::InsufficientBytes(self.buf_occupancy))
		}
	}
}
