use std::str::FromStr;
use sui::client_commands::{WalletContext, call_move};
use sui_json::SuiJsonValue;
use sui_types::TypeTag;
use sui_types::base_types::ObjectID;
use anyhow::{Result, anyhow};
use dotenv::dotenv;
use tokio::time;
use binance::api::Binance;
use binance::market::Market;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let package_id = dotenv::var("PACKAGE_ID")?;
    let coin_type = dotenv::var("COIN_TYPE")?;
    let admin_cap_id = dotenv::var("ADMIN_CAP_ID")?;
    let oracle_id = dotenv::var("ORACLE_ID")?;

    println!("Package ID: {}", &package_id);
    println!("Coin Type: {}", &coin_type);
    println!("AdminCap ID: {}", &admin_cap_id);
    println!("Oracle ID: {}", &oracle_id);

    let sui_config_path = dirs::home_dir()
        .expect("Failed to get home dir")
        .join(".sui").join("sui_config").join("client.yaml");
    println!("sui client config: {:?}", &sui_config_path);

    let mut wallet = WalletContext::new(&sui_config_path, None)
        .await
        .or(Err(anyhow!("Failed to get wallet context")))?;
    let active_address = wallet
        .active_address()
        .or(Err(anyhow!("Failed to get active address")))?;
    println!("active address: {:?}", active_address);

    let package_id = ObjectID::from_hex_literal(&package_id)?;
    let coin_type = TypeTag::from_str(&coin_type)?;
    let admin_cap_id = ObjectID::from_hex_literal(&admin_cap_id)?;
    let oracle_id = ObjectID::from_hex_literal(&oracle_id)?;

    let mut interval = time::interval(time::Duration::from_secs(10));
    let market: Market = Binance::new(None, None);

    loop {
        interval.tick().await;
        match market.get_price("BTCUSDT").await {
            Ok(resp) => {
                let price_u64 = (resp.price * 1_000.0) as u64;
                match call_move(
                    package_id,
                    "oracle",
                    "update_price",
                    vec![coin_type.clone()],
                    None,
                    10_000_000,
                    vec![
                        SuiJsonValue::from_object_id(admin_cap_id),
                        SuiJsonValue::from_object_id(oracle_id),
                        SuiJsonValue::new(price_u64.into())?,
                    ],
                    &mut wallet
                ).await {
                    Ok(_) => println!("{}", price_u64),
                    Err(e) => println!("{}", e),
                };
            },
            Err(e) => println!("{}", e),
        };
    }
}