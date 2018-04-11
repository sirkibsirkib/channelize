#Channelize

##Purpose
This project was created to provide an easy means of sending serializable structures over a `TcpStream`. Conceptually, the user provides their endpoint and wraps it with the provided structure `Endpoint`, which contains a small state machine that abstracts away from the bytes. These `Enpoint` structures each contain an internal buffer that grows as needed, and will handle structs with unknown serialized size. 

Although the intial intention was to only wrap a `std::net::TcpStream`, the interface was generalized to cover anything with the `io::{Read, Write}` traits. This will faciliate the use of other options, such as the TokIO's `TcpStream`, for example. 

##Comparing to `wire`
There already exists a crate with a similar purpose, by contributor 'TyOverby'
https://crates.io/crates/wire

However, it hasn't seen recent maintenance, and thus depends on a very old version of `serde`. I've also taken the liberty to approach the problem using my (slighly different) API.

##Examples
See the tests (at https://github.com/sirkibsirkib/channelize/blob/master/src/tests.rs ) for examples.