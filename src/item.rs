// Dependencies
use serde::{Serialize, Deserialize};

use crate::Error;

/// All we want is the collectibleItemId.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CatalogItem {
    collectible_item_id: String
}

/// The request payload for `https://apis.roblox.com/marketplace-items/v1/items/details`
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct MarketplaceItemDetailsPayload {
    item_ids: Vec<String>
}

/// Types of creators.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub enum CreatorTypes {
    User,
    Group
}

/// An item.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Item {
    pub collectible_item_id: String,
    pub collectible_product_id: String,
    pub creator_type: CreatorTypes,
    pub item_target_id: u64,
    pub creator_id: u64,
    pub price: u64,
    pub quantity_limit_per_user: u64
}
impl Item {
    /// Gets item from collectible_item_id.
    pub fn get_from_item_id(collectible_item_id: String) -> Result<Self, Error> {
        Ok(reqwest::blocking::Client::new()
            .post("https://apis.roblox.com/marketplace-items/v1/items/details")
            .json(&MarketplaceItemDetailsPayload {
                item_ids: vec![collectible_item_id]
            })
            .send()?
            .json::<Self>()?)
    }

    /// Gets item from id.
    pub fn get_from_id(id: u64) -> Result<Self, Error> {
        Self::get_from_item_id(
            reqwest::blocking::get(format!("https://catalog.roblox.com/v1/catalog/items/{}/details?itemType=Asset", id))?
                .json::<CatalogItem>()?.collectible_item_id)
    }
}