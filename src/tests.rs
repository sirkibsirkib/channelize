// for convenience when testing
const DEBUG_PRINTING: bool = false;
macro_rules! dprintln {
	() => ();
	($fmt:expr) => (if DEBUG_PRINTING {dprintln!(expr)});
	($fmt:expr, $($arg:tt)*) => (if DEBUG_PRINTING {
		print!(concat!($fmt, "\n"), $($arg)*)
	});
}

// import stdlib stuff
use ::std::{
	collections::HashSet,
	sync::Arc,
	io::{
		Error,
		ErrorKind,
	},
	net::{
		TcpListener,
		TcpStream,
	},
	thread,
};


// ----------------------------
// useful example code snippets
// ----------------------------
use super::{
	Endpoint,
	Message,
};

// define the structure we want to parse
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum TestMsg {
	A,
	B(u32),
	C(Vec<i8>),
}
impl Message for TestMsg {}

// this server expects TestMsg structs and bounces them back
fn echo_server(listener: TcpListener) -> Result<(), Error> {
	for stream in listener.incoming() {
		let mut endpt: Endpoint<_, TestMsg> = Endpoint::new(stream?);
		thread::spawn(move || { // move endpt inside closure
			while let Ok(m) = endpt.recv() {
				endpt.send(&m);
			}
		});
	}
	Ok(())
}

#[test]
fn example() {
	let (listener, addr) = bind_to_a_port().expect("oh no");
	use self::TestMsg::*;
	let h = thread::spawn(move || {
		//server
		let (stream, _addr) = listener.accept().expect("accept failed");
		let mut se: Endpoint<_, TestMsg> = Endpoint::new(stream);
		se.send(&A).is_ok();
		se.send(&C(vec![1,2,3])).is_ok();
		assert_eq!(se.recv().unwrap(), B(42));
		assert_eq!(se.recv().unwrap(), B(22));
	});
	//client
	let stream = TcpStream::connect(addr).expect("connect failed");
	let mut cl: Endpoint<_, TestMsg> = Endpoint::new(stream);
	let msgs = vec![B(42), B(22)];
	let (num_sent, res) = cl.send_all(msgs.iter());
	assert_eq!(num_sent, 2);
	res.is_ok();
	assert_eq!(cl.recv().unwrap(), A);
	assert_eq!(cl.recv().unwrap(), C(vec![1,2,3]));
	h.join().is_ok();
}


struct TestMsg2 {
	x: String,
	y: HashSet<char>,
}

#[test]
fn different_structs() {
	let (listener, addr) = bind_to_a_port().expect("bind fail");
	thread::spawn(move || {
		let stream = listener.accept().unwrap(); // just one client
		let mut endpt: Endpoint<_, TestMsg> = Endpoint::new(stream);
		while let Ok(m) = endpt.recv() {
			endpt.send(&m);
		}
	});
}

// ----------------------------------------

#[test]
fn a1_server() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![A,A,A,A],
			client_sends: vec![],
		},
		1,
	);
}

#[test]
fn echoes() {
	let (listener, addr) = bind_to_a_port().expect("bind fail");
	thread::spawn(move || {
		echo_server(listener);
	});

	use self::TestMsg::*;
	let messages = vec![A, B(1), B(4), C(vec![1,2,3]), A, C(vec![0,0]), A, A];
	let t = TcpStream::connect(addr).expect("connect fail");
	let mut endpt: Endpoint<_, TestMsg> = Endpoint::new(t);
	for m in messages.iter() {
		endpt.send(m);
		assert_eq!(&endpt.recv().unwrap(), m);
	}
}

#[test]
fn a1_client() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![],
			client_sends: vec![A,A,A],
		},
		1,
	);
}

#[test]
fn a1() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![A,A],
			client_sends: vec![A,A,A],
		},
		1,
	);
}

#[test]
fn a2() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![A,A],
			client_sends: vec![A,A,A],
		},
		2,
	);
}

#[test]
fn a5() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![A,A],
			client_sends: vec![A,A,A],
		},
		5,
	);
}

#[test]
fn b1() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![B(1), B(2)],
			client_sends: vec![B(4), B(8), B(9)],
		},
		1,
	);
}

#[test]
fn b2() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![B(1), B(2)],
			client_sends: vec![B(4), B(8), B(9)],
		},
		2,
	);
}

#[test]
fn ab1() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![A, A, B(1), B(2), A],
			client_sends: vec![B(4), A, B(8), A, B(9)],
		},
		2,
	);
}

#[test]
fn c1() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![C(vec![1,2,3])],
			client_sends: vec![C(vec![5,6]), C(vec![11,13,22,33])],
		},
		1,
	);
}

#[test]
fn c2() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![C(vec![1,2,3])],
			client_sends: vec![C(vec![5,6]), C(vec![11,13,22,33])],
		},
		2,
	);
}

#[test]
fn abc1() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![A, B(99), A, C(vec![1,2,3]), B(71)],
			client_sends: vec![C(vec![5,6]), A, A, C(vec![11,13,22,33]), B(22), B(4)],
		},
		1,
	);
}

#[test]
fn abc5() {
	use self::TestMsg::*;
	simple_test(
		"server_only_as",
		TestExchange {
			server_sends: vec![A, B(99), A, C(vec![1,2,3]), B(71)],
			client_sends: vec![C(vec![5,6]), A, A, C(vec![11,13,22,33]), B(22), B(4)],
		},
		2,
	);
}

fn simple_test(test_name: &'static str, te: TestExchange, num_clients: u32) {
	let (listener, addr) = bind_to_a_port().expect("fale");
	dprintln!("bound to {:?}", (&listener, &addr));
	let te = Arc::new(te);
	let mut handles = vec![];
	for client_id in 1..(num_clients+1) {
		let te2 = te.clone();
		let my_name = format!("{} (client {}/{})\t", test_name, client_id, num_clients);
		let h = thread::spawn(move || {
			client(&my_name, addr, te2).is_ok();
		});
		handles.push(h);
	}
	server(test_name, listener, te, num_clients).is_ok();
	for h in handles {
		h.join().is_ok();
	}
}

#[derive(Clone, Debug)]
struct TestExchange {
	server_sends: Vec<TestMsg>,
	client_sends: Vec<TestMsg>,
}

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
fn bind_to_a_port() -> Result<(TcpListener, SocketAddr), ()> {
	for port in 7000..12000 {
		let addr = localhost_with_port(port);
		if let Ok(listener) = TcpListener::bind(addr) {
			return Ok((listener, addr));
		}
	}
	Err(())
}

fn localhost_with_port(port: u16) -> SocketAddr {
	SocketAddr::new(IpAddr::V4(
		Ipv4Addr::new(127, 0, 0, 1)
	), port)
}

type Te = Arc<TestExchange>;

fn server(test_name: &str, listener: TcpListener, te: Te, num_clients: u32) -> Result<(), Error> {
	let mut handles = vec![];
 	for i in 1..(num_clients+1) {
 		let (stream, _) = listener.accept().expect("maymay");
		let endpoint: Endpoint<TcpStream, TestMsg> = Endpoint::new(stream);
		let te2 = te.clone();
		let name2 = format!("{} (server {}/{})", test_name, i, num_clients);
		let h = thread::spawn(move || {
			perform_exchange(
		    	&name2,
		    	endpoint,
		    	te2,
		    	true,
		    );
		});
		handles.push(h);
 	}
 	for h in handles {
 		if h.join().is_err() {
 			return Err(Error::new(ErrorKind::Other, "oh no!"))
 		}
 	}
	Ok(())
}


fn client(name: &str, addr: SocketAddr, te: Te) -> Result<(), Error> {
	let stream = TcpStream::connect(addr)?;
	let endpoint: Endpoint<TcpStream, TestMsg> = Endpoint::new(stream);
    perform_exchange(
    	name,
    	endpoint,
    	te,
    	false,
    );
	Ok(())
}


fn perform_exchange(
	name: &str,
	mut endpoint: Endpoint<TcpStream, TestMsg>,
	te: Te,
	is_server: bool,
) {
	let (incoming, outgoing) = if is_server {
		(&te.client_sends, &te.server_sends)
	} else {
		(&te.server_sends, &te.client_sends)
	};
	dprintln!("{} exchanging. expect {:?}, will send {:?}", name, &incoming, &outgoing);
	let (inlen, outlen) = (incoming.len(), outgoing.len());
	for (i,e) in outgoing.iter().enumerate() {
		endpoint.send(e).is_ok();
		dprintln!("{} sent   [{}/{}]: {:?}", name, i+1, outlen, e);
	}
	for (i,e) in incoming.iter().enumerate() {
		dprintln!("{} recv'd [{}/{}]: {:?}", name, i+1, inlen, e);
		assert_eq!(e, &endpoint.recv().expect("danky"));
	}
	dprintln!(">> finished exchange for {}", name);
}