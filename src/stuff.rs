
use ::std::{
	io,
	fmt::Debug,
	convert::From,
	marker::PhantomData,
	net::TcpStream,
};

use ::bincode;
use byteorder::{
	LittleEndian,
	ReadBytesExt,
	WriteBytesExt,
};
use serde::{
	Serialize,
	de::DeserializeOwned,
};



pub trait Stream: io::Read + io::Write + Sized {
	fn set_nonblocking(&mut self, value: bool) -> Result<(), io::Error>;
}
impl Stream for TcpStream {
	fn set_nonblocking(&mut self, value: bool) -> Result<(), io::Error> {
		TcpStream::set_nonblocking(self, value)
	}
}



pub trait Message: Serialize + DeserializeOwned + Sized + Debug {}



#[derive(Debug)]
pub enum RecvError {
	NoBytes,
	InsufficientBytes(usize),
	SocketErr(io::Error)
}



#[derive(Debug)]
pub struct Endpoint<T,M>
where
	T: Stream,
	M: Message,
{
	stream: T,
	buf: Vec<u8>,
	buf_occupancy: usize,
	payload_bytes: Option<u32>,
	phantom: PhantomData<M>,
}


impl From<io::Error> for RecvError {
	fn from(t: io::Error) -> RecvError {
		RecvError::SocketErr(t)
	}
}


const LEN_BYTES: usize = 4; 


impl<'m,T,M> Endpoint<T,M>
where
	T: Stream,
	M: Message + 'm,
{
	pub fn new(mut stream: T) -> Endpoint<T,M> {
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
		self.buf.resize(capacity, 0u8);
	}

	pub fn current_buffer_size(&mut self) -> usize {
		self.buf_occupancy
	}

	pub fn into_inner(mut self) -> (T, Vec<u8>) {
		self.squash_buffer();
		(self.stream, self.buf)
	}

	pub fn squash_buffer(&mut self) -> usize {
		self.buf.resize(self.buf_occupancy, 0u8);
		self.buf.shrink_to_fit();
		self.buf_occupancy
	}

	pub fn drain_nonblocking(&mut self) -> Vec<M> {
		let mut v = vec![];
		while let Ok(m) = self.try_recv() {
			v.push(m);
		}
		v
	}

	pub fn send_all<I>(&mut self, m_iter: I) -> (usize, Result<(), io::Error>)
	where I: Iterator<Item = &'m M> {
		let mut tot_sent = 0;
	    for m in m_iter {
	    	match self.send(m) {
	    		Ok(_) => tot_sent += 1,
	    		Err(e) => return (tot_sent, Err(e)),
	    	}
	    }
	    (tot_sent, Ok(()))
	}

	pub fn send(&mut self, m: &M) -> Result<usize, io::Error> {
		let encoded: Vec<u8> = bincode::serialize(&m).expect("nawww");
		let len = encoded.len();
		if len > ::std::u32::MAX as usize {
			panic!("`send()` can only handle payloads up to std::u32::MAX");
		}
		let mut encoded_len = vec![];
		encoded_len.write_u32::<LittleEndian>(len as u32)?;
		self.stream.write(&encoded_len)?;
		self.stream.write(&encoded)?;
		Ok(len)
	}

	pub fn new_change_type<Q: Message>(self) -> Endpoint<T,Q> {
		Endpoint {
			stream: self.stream,
			buf: self.buf,
			buf_occupancy: self.buf_occupancy,
			payload_bytes: self.payload_bytes,
			phantom: PhantomData,
		}
	}

	pub fn recv(&mut self) -> Result<M, RecvError> {
		self.stream.set_nonblocking(false)?;
		loop {
			match self.try_recv() {
				Err(RecvError::NoBytes) => (),
				Err(RecvError::InsufficientBytes(_)) => (),
				Ok(m) => {
					self.stream.set_nonblocking(true)?;
					return Ok(m)
				}
				Err(x) => return Err(x),
			}
		}
	}

	pub fn try_recv(&mut self) -> Result<M, RecvError> {
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
			Err(RecvError::NoBytes)
		} else {
			Err(RecvError::InsufficientBytes(self.buf_occupancy))
		}
	}
}


//////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////
//Vers 2

use std::collections::VecDeque;

#[derive(Debug)]
pub struct ReadState<M> where M: Message {
	buf: Vec<u8>,
	buf_occupancy: usize,
	payload_bytes: Option<u32>,
	phantom: PhantomData<M>,
}

impl<M> ReadState<M> where M: Message {
	const LEN_BYTES: usize = 4; 

	pub fn new() -> ReadState<T,M> {
		stream.set_nonblocking(true)
		.expect("Failed to set given socket to non-blocking mode!");
		ReadState {
			buf: vec![],
			buf_occupancy: 0,
			payload_bytes: None,
			phantom: PhantomData,
		}
	}

	fn ensure_buf_capacity(&mut self, capacity: usize) {
		self.buf.resize(capacity, 0u8);
	}

	pub fn current_buffer_size(&mut self) -> usize {
		self.buf_occupancy
	}

	pub fn squash_buffer(&mut self) -> usize {
		self.buf.resize(self.buf_occupancy, 0u8);
		self.buf.shrink_to_fit();
		self.buf_occupancy
	}

	pub fn drain_nonblocking<R: io::Read>(&mut self, r: &mut R) -> Vec<M> {
		let mut v = vec![];
		while let Ok(m) = self.try_recv(r) {
			v.push(m);
		}
		v
	}

	pub fn recv<R: io::Read>(&mut self, r: &mut R) -> Result<M, RecvError> {
		self.stream.set_nonblocking(false)?;
		loop {
			match self.try_recv(r) {
				Err(RecvError::NoBytes) => (),
				Err(RecvError::InsufficientBytes(_)) => (),
				Ok(m) => {
					self.stream.set_nonblocking(true)?;
					return Ok(m)
				}
				Err(x) => return Err(x),
			}
		}
	}

	pub fn try_recv<R: io::Read>(&mut self, r: &mut R) -> Result<M, RecvError> {
		if self.payload_bytes.is_none() {
			self.ensure_buf_capacity(Self::LEN_BYTES);
			self.buf_occupancy +=
				r.read(&mut self.buf[self.buf_occupancy..Self::LEN_BYTES])?;
			if self.buf_occupancy == 4 {
				self.payload_bytes = Some(
					(&self.buf[0..Self::LEN_BYTES]).read_u32::<LittleEndian>()
					.expect("naimen")
				);
			}
		}
		if let Some(pb) = self.payload_bytes {
			// try to get the payload bytes
			let buf_end: usize = Self::LEN_BYTES + pb as usize;
			self.ensure_buf_capacity(buf_end);
			self.buf_occupancy +=
				r.read(&mut self.buf[Self::LEN_BYTES..buf_end])?;

			if self.buf_occupancy == buf_end {
				// read message to completion!
				let decoded: M = bincode::deserialize(
					&self.buf[Self::LEN_BYTES..buf_end]
				).expect("jirre");
				self.buf_occupancy = 0;
				self.payload_bytes = None;
				return Ok(decoded);
			}
		}
		if self.buf_occupancy == 0 {
			Err(RecvError::NoBytes)
		} else {
			Err(RecvError::InsufficientBytes(self.buf_occupancy))
		}
	}
}







fn send<M>(medium &mut io::Write, m: &M) where M: Message -> Result<usize, io::Error> {
	let encoded: Vec<u8> = bincode::serialize(&m).expect("nawww");
	let len = encoded.len();
	if len > ::std::u32::MAX as usize {
		panic!("`send()` can only handle payloads up to std::u32::MAX");
	}
	let mut encoded_len = vec![];
	encoded_len.write_u32::<LittleEndian>(len as u32)?;
	self.stream.write(&encoded_len)?;
	self.stream.write(&encoded)?;
	Ok(len)
}



enum Testy {
	A,
	B(u32),
	C(Vec<u8>),
}

fn test() {
	let addr = "127.0.0.1:8008";
	let listener = TcpListener::bind(addr).unwrap();

	let h = thread::spawn(move || {
		let a = listener.accept().unwrap();
		let rx = ReadState::new::<Testy>();

		send(&mut a, Testy::A);
		while let Some(m) = rx.recv(&mut a) {
			send(&mut a, Testy::A);
		}

		// let bx = WrappedBoth::new(a);
		// if let Some(m) = bx.recv() {
		// 	bx.send(Testy::A);
		// }
	});

	{
		let mut togo = 10;
		let b = TcpStream::connect(addr).unwrap();
		let rx = ReadState::new::<Testy>();

		while togo > 0 {
			if let Some(m) = rx.recv(&mut a) {
				send(&mut a, Testy::B(2));
			}
			togo -= 1;
		}
	}
	h.join().is_ok();
}




/*
trait Sender {
	send()
	send_all()
}
impl by:
	WrappedSender
	WrappedPair
	WrappedBidir



trait receiver {
	try_recv()
	recv()
}
impl by:
	WrappedReceiver
	WrappedPair
	WrappedBidir


struct WrappedPair<R, W> where R: io::Read, W: io::Write {
	r: R,
	w: W,
}

struct WrappedBidir<B> where B: io::Read + io::Write {
	b: B,
}
*/
	