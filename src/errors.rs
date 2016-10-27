use ::std::io;
use ::std::net;
use hyper;
use serde_json;

error_chain! {
     foreign_links {
         io::Error, Io;
         net::AddrParseError, Address;
         hyper::Error, Hyper;
         serde_json::Error, Json;
     }
}
