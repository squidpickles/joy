#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate iron;
extern crate hyper;
extern crate logger;
extern crate time;
extern crate serde;
extern crate serde_json;
extern crate handlebars_iron;

mod errors;

use iron::prelude::*;
use iron::status;
use iron::Handler;
use iron::AfterMiddleware;
use iron::headers::{HttpDate, CacheControl, CacheDirective, LastModified};
use logger::Logger;
use std::error::Error;
use handlebars_iron::{HandlebarsEngine, DirectorySource, Template};
use hyper::server::Listening;
use hyper::client::Client;
use std::collections::BTreeMap;

const JOY_URL: &'static str = "https://raw.githubusercontent.com/squidpickles/slippybot/master/joy.json";
const LISTEN_ADDR: &'static str = "127.0.0.1:4707";
const CACHE_TIMEOUT: u32 = 300; // seconds

struct JoyHandler;
impl JoyHandler {
    pub fn new() -> JoyHandler {
        JoyHandler {}
    }

    fn fetch_joy(&self) -> Result<BTreeMap<String, Vec<String>>, errors::Error> {
        let client = Client::new();
        let response = try!(client.get(JOY_URL).send());
        let joy = try!(serde_json::from_reader(response));
        Ok(joy)
    }

    fn set_cache_headers(&self, res: &mut Response, date: time::Tm) {
        res.headers.set(CacheControl(vec![
            CacheDirective::MaxAge(CACHE_TIMEOUT)
        ]));
        res.headers.set(LastModified(HttpDate(date)));
    }
}

impl Handler for JoyHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let mut resp = Response::new();
        let joy = match self.fetch_joy() {
            Ok(jj) => jj,
            Err(err) => return Err(iron::IronError::new(err, status::InternalServerError))
        };
        let now = time::now_utc();
        resp.set_mut(Template::new("index", joy)).set_mut(status::Ok);
        self.set_cache_headers(&mut resp, now);
        Ok(resp)
    }
}

struct ErrorReporter;
impl AfterMiddleware for ErrorReporter {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        println!("{}", err.description());
        Err(err)
    }
}

pub struct WebServer {
    chain: Chain,
}

impl WebServer {

    pub fn new() -> Result<WebServer, errors::Error> {
        let mut chain = Chain::new(JoyHandler::new());

        let mut hbse = HandlebarsEngine::new();
        let source = Box::new(DirectorySource::new("./templates/", ".hbs"));
        hbse.add(source);
        try!(match hbse.reload() {
            Err(err) => Err(format!("Handlebars error: {}", err.description())),
            _ => Ok(())
        });
        chain.link_after(hbse);

        let (logger_before, logger_after) = Logger::new(None);
        chain.link_before(logger_before);
        chain.link_after(logger_after);

        chain.link_after(ErrorReporter);

        Ok(WebServer { chain: chain })
    }

    pub fn run(self, listen_address: &str) -> Result<Listening, errors::Error> {
        let iron = Iron::new(self.chain);
        iron.http(listen_address).map_err(|e| e.into())
    }
}

pub fn main() {
    let server = WebServer::new().unwrap();
    println!("Listening on {}", LISTEN_ADDR);
    server.run(LISTEN_ADDR).unwrap();
}
