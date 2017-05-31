use ::std::io;
use ::std::net;
use hyper;
use serde_json;

error_chain! {
     foreign_links {
         Io(io::Error);
         Address(net::AddrParseError);
         Hyper(hyper::Error);
         Json(serde_json::Error);
     }
}
