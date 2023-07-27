// Dependencies
use std::{path::PathBuf, time::{SystemTime, Duration}};
use atomic_counter::{RelaxedCounter, AtomicCounter};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;

use crate::Error;

/// The default check url for proxies.
const CHECK_URL: &str = "https://google.com/";

/// Loads all of the contents from a file and returns its lines.
pub fn load_file_lines(path: PathBuf) -> Result<Vec<String>, std::io::Error> {
    Ok(std::fs::read_to_string(path)?.lines().map(|x| x.to_string()).collect())
}

/// Checks if a proxy is working (blocking).
pub fn check_proxy(proxy: &str, check_url: Option<&str>) -> Result<bool, reqwest::Error> {
    Ok(
        reqwest::blocking::Client::builder()
            .proxy(reqwest::Proxy::all(proxy)?)
            .build()?
            .get(check_url.unwrap_or(CHECK_URL))
            .send()
            .is_ok()
    )
}

/// Checks if many proxies work.
pub fn check_proxies(proxy_list: Vec<String>, check_url: Option<&str>) -> Vec<String> {
    // Default the check url
    let check_url = check_url.unwrap_or(CHECK_URL);

    // Iterate through each concurrently and return the result
    proxy_list
        .into_par_iter()
        .filter(|x| check_proxy(x, Some(check_url)).unwrap_or(false))
        .collect()
}

/// Represents a proxy.
#[derive(Default, Debug)]
pub struct Proxy {
    /// The proxy url
    url: String,
    /// A client used to send requests.
    http: reqwest::blocking::Client,
    /// This holds all of the requests in the past minute.
    /// Must be manually cleared and reset to 0.
    pub requests: RelaxedCounter,
    /// Whether on cooldown or not.
    pub cooldown: Option<SystemTime>
}
impl Proxy {
    /// Creates a new instance.
    pub fn new(url: String) -> Result<Self, reqwest::Error> {
        Ok(Self {
            url: url.clone(),
            http: reqwest::blocking::Client::builder()
                .proxy(reqwest::Proxy::all(url)?)
                .build()?,
            ..Default::default()
        })
    }

    /// Checks if the proxy is working.
    pub fn check(&self, check_url: Option<&str>) -> Result<bool, reqwest::Error> {
        Ok(
            reqwest::blocking::Client::builder()
                .proxy(reqwest::Proxy::all(self.url.clone())?)
                .build()?
                .get(check_url.unwrap_or(CHECK_URL))
                .send()
                .is_ok()
        )
    }

    /// Resets the requests counter.
    pub fn reset(&self) {
        self.requests.reset();
    }

    /// Sends a post request.
    pub fn send_post<T: Serialize + ?Sized>(&mut self, url: &str, data: &T, csrf: Option<String>) -> Result<reqwest::blocking::Response, Error> {
        // Stop if we are on cooldown
        if let Some(cooldown) = self.cooldown {
            if cooldown > SystemTime::now() {
                return Err(Error::OnCooldown)
            } else {
                self.cooldown = None
            }
        }

        // Create the request and send it
        let response = self.http.post(url)
            .json(data)
            .header("x-csrf-token", csrf.unwrap_or_default())
            .send()?;
        self.requests.inc();

        // Check for rate limit
        if response.status() == 429 {
            let delay = response.headers().get("Retry-After").unwrap().to_str().unwrap().parse().unwrap();
            self.cooldown = Some(SystemTime::now() + Duration::from_secs(delay));
            return Err(Error::RateLimited)
        }

        // Return the response
        Ok(response)
    }
}