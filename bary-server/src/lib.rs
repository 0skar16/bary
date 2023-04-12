use std::{collections::BTreeMap, io::{Read, Cursor}, path::PathBuf, fs::File};

use rocket::{Config as RocketConfig, error::LaunchError, Rocket, Handler, handler::Outcome, Route, http::{Method, ContentType}, response::Responder, Response};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use tar::Archive;
pub use rocket;
pub struct Server {
    rocket: Rocket,
    frontend: BTreeMap<String, Vec<u8>>,
    routes: BTreeMap<String, Vec<Route>>,
}
impl Server {
    pub fn new(port: u16, frontend: impl Frontend, secret_key: Option<impl Into<String>>) -> Server {
        let mut config = RocketConfig::build(rocket::config::Environment::Production);
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
        let mut routes = vec![];
        for (path, bytes) in self.frontend {
            let handler = VecHandler(VFResponder(bytes, PathBuf::from(&path)));
            if path.ends_with("index.html") {
                let path = path.trim_end_matches("index.html");
                let route = Route::new(Method::Get, path, handler.clone());
                routes.push(route);
            }
            let route = Route::new(Method::Get, path.as_str(), handler);
            routes.push(route);
        }
        rocket = rocket.mount("/", routes);
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
pub fn load_config(config_path: PathBuf) -> Result<Config> {
    let abs_path = std::fs::canonicalize(config_path)?;
    let f = File::open(abs_path)?;
    Ok(serde_yaml::from_reader(f)?)
}
pub fn load_config_from_bytes(config: Vec<u8>) -> Result<Config> {
    Ok(serde_yaml::from_slice(&config)?)
}
pub fn load_config_from_str(config: &str) -> Result<Config> {
    Ok(serde_yaml::from_str(config)?)
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub frontend: PathBuf,
    pub port: u16,
}


#[derive(Debug, Deserialize)]
pub struct BaryAppAttr {
    pub secret_key: String,
}