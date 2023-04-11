use std::{collections::BTreeMap, io::{Read, Cursor}, path::PathBuf};

use rocket::{Config, error::LaunchError, Rocket, Handler, handler::Outcome, Route, http::{Method, ContentType}, response::Responder, Response};
use anyhow::Result;
use tar::Archive;

pub struct Server {
    rocket: Rocket,
    frontend: BTreeMap<String, Vec<u8>>,
    routes: BTreeMap<String, Vec<Route>>,
}
impl Server {
    pub fn new(port: u16, frontend: impl Frontend, secret_key: Option<impl Into<String>>) -> Server {
        let mut config = Config::build(rocket::config::Environment::Production);
        config = config.port(port);
        if let Some(secret_key) = secret_key {
            config = config.secret_key(secret_key.into())
        }
        let rocket = rocket::custom(config.expect("Couldn't build rocket sconfig"));
        Server {
            rocket,
            frontend: frontend.resolve().expect("Couldn't resolve frontend"),
            routes: BTreeMap::new(),
        }
    }

    pub fn start(self) -> Result<(), LaunchError> {
        let mut rocket = self.rocket;
        for (path, bytes) in self.frontend {
            let handler = VecHandler(VFResponder(bytes, PathBuf::from(&path)));
            if path.ends_with("index.html") {
                let path = path.trim_end_matches("index.html");
                let route = Route::new(Method::Get, path, handler.clone());
                rocket = rocket.mount("/", vec![route])
            }
            let route = Route::new(Method::Get, path.as_str(), handler);
            let routes = vec![route];
            rocket = rocket.mount("/", routes);
        }
        for (base, routes) in self.routes {
            rocket = rocket.mount(&base, routes);
        }
        Err(rocket.launch())
    }
    pub fn mount<R: Into<Vec<Route>>>(&mut self, base: &str, routes: R) {
        self.routes.insert(base.to_string(), routes.into());
    }
}
pub trait Frontend {
    fn resolve(self) -> Result<BTreeMap<String, Vec<u8>>>;
}
impl<R: Read + Sized> Frontend for Archive<R> {
    fn resolve(mut self) -> Result<BTreeMap<String, Vec<u8>>> {
        let mut files = BTreeMap::new();
        for entry in self.entries()? {
            if let Ok(mut entry) = entry {
                if entry.header().entry_type().is_file() {
                    let mut buf = Vec::new();
                    entry.read_to_end(&mut buf)?;
                    let path = format!("/{}", entry.header().path()?.display());
                    files.insert(path, buf);
                }
                
            }
        }
        Ok(files)
    }
}
#[derive(Clone, Debug)]
pub struct VecHandler(pub VFResponder);
impl Handler for VecHandler {
    fn handle<'r>(&self, request: &'r rocket::Request, _data: rocket::Data) -> rocket::handler::Outcome<'r> {
        Outcome::from(request, self.0.clone())

    }
}
#[derive(Debug, Clone)]
pub struct VFResponder(pub Vec<u8>, pub PathBuf);

impl<'r> Responder<'r> for VFResponder {
    fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'r> {
        let mut builder = Response::build();
        let mut builder = builder.raw_body(rocket::response::Body::Sized(Cursor::new(self.0.clone()), self.0.len() as u64));
        if let Some(ext) = self.1.extension() {
            if let Some(ct) = ContentType::from_extension(&ext.to_string_lossy()) {
                builder = builder.header(ct);
            }
        }
        builder.ok()
    }
}