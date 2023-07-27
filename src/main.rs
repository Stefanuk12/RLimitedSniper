// Dependencies
pub mod user;
pub mod proxy;
pub mod item;

/// Possible errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("missing x-csrf-token from headers")]
    MissingCSRF,
    #[error("rate limited")]
    RateLimited,
    #[error("on cooldown")]
    OnCooldown,
    #[error("cannot afford item")]
    Broke,
    #[error("an unknown error occured")]
    Other(String)
}

fn main() {
    // Load cookies

    // Load proxies

    // do some other stuff
}