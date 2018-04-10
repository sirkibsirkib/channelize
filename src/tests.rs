
use super::{
	Endpoint,
	Message,
	// EE,
};

// import stdlib stuff
use ::std::{
	sync::Arc,
	// io,
	// io::prelude::*,
	// time,
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

// use serde::{
// 	Serialize,
// 	Deserialize,
// 	de::DeserializeOwned,
// };

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum TestMsg {
	A,
	B(u32),
	C(Vec<i8>),
}
impl Message for TestMsg {}


#[test]
fn server_only_as() {
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

// #[test]
// fn client_only_as() {
// 	use self::TestMsg::*;
// 	simple_test(
// 		"client_only_as",
// 		TestExchange {
// 			server_sends: vec![],
// 			client_sends: vec![A,A,A,A],
// 		},
// 	);
// }

// #[test]
// fn both_as() {
// 	use self::TestMsg::*;
// 	simple_test(
// 		"both_as",
// 		TestExchange {
// 			server_sends: vec![A,A,A,A,A,A],
// 			client_sends: vec![A,A,A,A],
// 		},
// 	);
// }

fn simple_test(test_name: &'static str, te: TestExchange, num_clients: u32) {
	let (listener, addr) = bind_to_a_port().expect("fale");
	println!("bound to {:?}", (&listener, &addr));
	let te = Arc::new(te);
	let mut handles = vec![];
	for client_id in 0..num_clients {
		let te2 = te.clone();
		let h = thread::spawn(move || {
			client(test_name, addr, te2).is_ok();
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

fn server(name: &str, listener: TcpListener, te: Te, num_clients: u32) -> Result<(), Error> {
	let mut handles = vec![];
	let name = Arc::new(name.to_owned());
 	for i in 0..num_clients {
 		let (stream, _) = listener.accept().expect("maymay");
		let endpoint: Endpoint<TestMsg> = Endpoint::new(stream);
		let te2 = te.clone();
		let name2 = name.clone();
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
	let endpoint: Endpoint<TestMsg> = Endpoint::new(stream);
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
	mut endpoint: Endpoint<TestMsg>,
	te: Te,
	is_server: bool,
) {
	let (incoming, outgoing) = if is_server {
		(&te.client_sends, &te.server_sends)
	} else {
		(&te.server_sends, &te.client_sends)
	};
	println!("{} exchanging. expect {:?}, will send {:?}", name, &incoming, &outgoing);
	let (inlen, outlen) = (incoming.len(), outgoing.len());
	for (i,e) in outgoing.iter().enumerate() {
		endpoint.send(e).is_ok();
		println!("{} sent   [{}/{}]: {:?}", name, i+1, outlen, e);
	}
	for (i,e) in incoming.iter().enumerate() {
		println!("{} recv'd [{}/{}]: {:?}", name, i+1, inlen, e);
		assert_eq!(e, &endpoint.recv().expect("danky"));
	}
	println!(">> finished exchange for {}", name);
}