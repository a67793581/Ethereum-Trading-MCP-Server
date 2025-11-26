use ethers::prelude::*;
use rmcp::{tool};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

// ABIs
const ERC20_ABI: &str = r#"[
    {"constant":true,"inputs":[{"name":"_owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"type":"function"},
    {"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"type":"function"}
]"#;
const CHAINLINK_ABI: &str = r#"[
    {"constant":true,"inputs":[],"name":"latestRoundData","outputs":[{"name":"roundId","type":"uint80"},{"name":"answer","type":"int256"},{"name":"startedAt","type":"uint256"},{"name":"updatedAt","type":"uint256"},{"name":"answeredInRound","type":"uint80"}],"type":"function"},
    {"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"type":"function"}
]"#;
const UNISWAP_V2_ROUTER_ABI: &str = r#"[
    {"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"}],"name":"getAmountsOut","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"view","type":"function"},
    {"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"}
]"#;

// Addresses
const UNISWAP_V2_ROUTER_ADDRESS: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";

fn create_provider() -> Result<Provider<Http>, String> {
    let infura_project_id = env::var("INFURA_PROJECT_ID").map_err(|_| "INFURA_PROJECT_ID not set".to_string())?;
    let rpc_url = format!("https://mainnet.infura.io/v3/{}", infura_project_id);
    Provider::<Http>::try_from(rpc_url).map_err(|e| e.to_string())
}

fn get_token_addresses() -> HashMap<&'static str, &'static str> {
    let mut token_addresses = HashMap::new();
    token_addresses.insert("WETH", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    token_addresses.insert("DAI", "0x6B175474E89094C44Da98b954EedeAC495271d0F");
    token_addresses.insert("USDC", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    token_addresses
}

#[tool]
pub async fn get_balance(wallet_address: String, token_contract_address: Option<String>) -> Result<serde_json::Value, String> {
    let provider = create_provider()?;
    let wallet_address: Address = wallet_address.parse().map_err(|e| e.to_string())?;

    let balance = if let Some(token_address) = token_contract_address {
        let token_address: Address = token_address.parse().map_err(|e| e.to_string())?;
        let contract = Contract::new(token_address, serde_json::from_str(ERC20_ABI).unwrap(), Arc::new(provider));

        let balance: U256 = contract.method("balanceOf", wallet_address).unwrap().call().await.map_err(|e| e.to_string())?;
        let decimals: u8 = contract.method("decimals", ()).unwrap().call().await.map_err(|e| e.to_string())?;

        let balance_decimal = Decimal::from_u128(balance.as_u128()).unwrap();
        balance_decimal / Decimal::from_u128(10u128.pow(decimals as u32)).unwrap()
    } else {
        let balance = provider.get_balance(wallet_address, None).await.map_err(|e| e.to_string())?;
        let balance_decimal = Decimal::from_u128(balance.as_u128()).unwrap();
        balance_decimal / Decimal::from_u128(10u128.pow(18)).unwrap() // ETH has 18 decimals
    };

    Ok(json!({ "balance": balance.to_string() }))
}

#[tool]
pub async fn get_token_price(token_symbol: String) -> Result<serde_json::Value, String> {
    let provider = create_provider()?;
    let mut price_feeds = HashMap::new();
    price_feeds.insert("ETH", "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419"); // ETH/USD
    price_feeds.insert("DAI", "0xAed0c38402a5d19df6E4c03F4E2DceD6e29c1ee9"); // DAI/USD
    price_feeds.insert("USDC", "0x8fFfFfd4AfB6115b954Fe326c6b940E56d66d450"); // USDC/USD

    let feed_address = price_feeds.get(token_symbol.as_str()).ok_or("Unsupported token symbol")?;
    let feed_address: Address = feed_address.parse().unwrap();

    let contract = Contract::new(feed_address, serde_json::from_str(CHAINLINK_ABI).unwrap(), Arc::new(provider));
    let (_round_id, answer, _started_at, _updated_at, _answered_in_round): (U256, I256, U256, U256, U256) =
        contract.method("latestRoundData", ()).unwrap().call().await.map_err(|e| e.to_string())?;
    let decimals: u8 = contract.method("decimals", ()).unwrap().call().await.map_err(|e| e.to_string())?;

    let price = Decimal::from_i128(answer.as_i128()).unwrap() / Decimal::from_u128(10u128.pow(decimals as u32)).unwrap();

    Ok(json!({ "price_usd": price.to_string() }))
}

#[tool]
pub async fn swap_tokens(from_token: String, to_token: String, amount: String, slippage_tolerance: f64) -> Result<serde_json::Value, String> {
    let provider = create_provider()?;
    let private_key = env::var("PRIVATE_KEY").map_err(|_| "PRIVATE_KEY not set".to_string())?;
    let wallet = private_key.parse::<LocalWallet>().map_err(|e| e.to_string())?;
    let client = SignerMiddleware::new(provider.clone(), wallet.with_chain_id(Chain::Mainnet));
    let client = Arc::new(client);

    let token_addresses = get_token_addresses();
    let from_token_address = token_addresses.get(from_token.as_str()).ok_or("Unsupported from_token")?.parse::<Address>().unwrap();
    let to_token_address = token_addresses.get(to_token.as_str()).ok_or("Unsupported to_token")?.parse::<Address>().unwrap();

    let from_contract = Contract::new(from_token_address, serde_json::from_str(ERC20_ABI).unwrap(), client.clone());
    let decimals: u8 = from_contract.method("decimals", ()).unwrap().call().await.map_err(|e| e.to_string())?;
    
    let amount_decimal = Decimal::from_str(&amount).map_err(|e| e.to_string())?;
    let amount_in = U256::from_dec_str(&(amount_decimal * Decimal::from_u128(10u128.pow(decimals as u32)).unwrap()).to_string()).unwrap();

    let router_address = UNISWAP_V2_ROUTER_ADDRESS.parse::<Address>().unwrap();
    let router = Contract::new(router_address, serde_json::from_str(UNISWAP_V2_ROUTER_ABI).unwrap(), client.clone());

    let path = vec![from_token_address, to_token_address];
    let amounts_out: Vec<U256> = router.method("getAmountsOut", (amount_in, path.clone())).unwrap().call().await.map_err(|e| e.to_string())?;
    let amount_out_min = amounts_out[1] * U256::from((1.0 - slippage_tolerance) * 1000.0) / U256::from(1000);

    let deadline = U256::from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 60 * 20); // 20 minutes from now

    // Build the transaction
    let tx = router.method("swapExactTokensForTokens", (amount_in, amount_out_min, path, client.address(), deadline)).unwrap();
    
    // Simulate the transaction to get gas estimate
    let estimated_gas = tx.estimate_gas().await.map_err(|e| e.to_string())?;
    let gas_price = provider.get_gas_price().await.map_err(|e| e.to_string())?;
    let estimated_cost = estimated_gas * gas_price;

    let to_contract = Contract::new(to_token_address, serde_json::from_str(ERC20_ABI).unwrap(), client.clone());
    let to_decimals: u8 = to_contract.method("decimals", ()).unwrap().call().await.map_err(|e| e.to_string())?;
    let amount_out_decimal = Decimal::from_u128(amounts_out[1].as_u128()).unwrap() / Decimal::from_u128(10u128.pow(to_decimals as u32)).unwrap();

    Ok(json!({
        "estimated_amount_out": amount_out_decimal.to_string(),
        "estimated_gas_cost_wei": estimated_cost.to_string()
    }))
}
