// Dependencies
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Serialize, Deserialize};
use crate::{Error, proxy::Proxy, item::{Item, CreatorTypes}};

/// Gets the CSRF token of a cookie.
pub fn get_csrf(cookie: &str) -> Result<String, Error> {
    Ok(reqwest::blocking::Client::default()
        .post("https://auth.roblox.com/v2/logout")
        .header("Cookie", &format!(".ROBLOSECURITY={};", cookie))
        .send()?
        .headers()
        .get("x-csrf-token")
        .ok_or(Error::MissingCSRF)?
        .to_str()
        .or(Err(Error::Other(String::from("unable to convert csrf to a string, wtf?"))))?
        .to_string())
}

/// Gets the CSRF token of many cookies.
pub fn get_csrfs(cookies: Vec<String>) -> Result<Vec<String>, Error> {
    Ok(cookies
        .into_par_iter()
        .map(|x| get_csrf(&x).ok())
        .while_some()
        .collect())
}

/// Data needed to purchase an item.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct BuyData {
    pub collectible_item_id: String,
    pub collectible_product_id: String,
    pub expected_currency: u8,
    pub expected_price: u64,
    pub expected_purchaser_id: u64,
    pub expected_purchaser_type: CreatorTypes,
    pub expected_seller_id: u64,
    pub expected_seller_type: CreatorTypes,
    pub idemptoency_key: String,
}
impl From<Item> for BuyData {
    fn from(value: Item) -> Self {
        Self {
            collectible_item_id: value.collectible_item_id,
            collectible_product_id: value.collectible_product_id,
            expected_currency: 1,
            expected_price: value.price,
            expected_purchaser_id: value.creator_id,
            expected_purchaser_type: CreatorTypes::User,
            expected_seller_id: 0,
            expected_seller_type: value.creator_type,
            idemptoency_key: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// The response from buying an item.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct PurchaseData {
    /// Can be one of the following:
    /// 
    /// - `Flooded`
    pub purchase_result: Option<String>,
    /// Can be one of the following:
    /// 
    /// - `QuantityExhausted`
    /// - `Only support User type for purchasing: request purchaser type Group`
    pub error_message: Option<String>,
    pub purchased: bool
}

/// Represents a user.
pub struct User {
    /// The cookie for this user.
    cookie: String,
    /// The user id of this user.
    user_id: u64,
    /// How much robux this user has.
    robux: u64,
    /// Purchase history of this session.
    /// Helps to ensure we do not buy the same item again for no reason.
    history: Vec<u64>
}
impl User {
    /// Gets the CSRF token of a cookie.
    pub fn get_csrf(&self) -> Result<String, Error> {
        Ok(reqwest::blocking::Client::default()
            .post("https://auth.roblox.com/v2/logout")
            .header("Cookie", &format!(".ROBLOSECURITY={};", self.cookie))
            .send()?
            .headers()
            .get("x-csrf-token")
            .ok_or(Error::MissingCSRF)?
            .to_str()
            .or(Err(Error::Other(String::from("unable to convert csrf to a string, wtf?"))))?
            .to_string())
    }

    /// Purchases an item.
    /// This function does no checks to see if we can afford it (except from what's within the cache of this user.)
    /// This is done to increase speed.
    pub fn purchase(&mut self, item: &BuyData, mut proxy: Proxy) -> Result<PurchaseData, Error> {
        // Check if we can afford
        if item.expected_price > self.robux {
            return Err(Error::Broke);
        }

        // Customise the buy data to apply to us
        let mut item = item.clone();
        item.expected_seller_id = self.user_id;

        // Send the request, could require a challenge
        let csrf = self.get_csrf()?;
        let response: PurchaseData = proxy
            .send_post(&format!("https://apis.roblox.com/marketplace-sales/v1/item/{}/purchase-item", item.collectible_item_id), &item, Some(csrf))?
            .json()?;

        // Check if it was successful
        if response.purchased {
            self.robux -= item.expected_price
        }

        // Return
        Ok(response)
    }
}