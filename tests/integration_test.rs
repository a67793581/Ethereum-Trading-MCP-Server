// Note: This is an integration test and requires a valid INFURA_PROJECT_ID in the .env file.

use mcp_server::{get_balance};
use serde_json::Value;

#[tokio::test]
async fn test_get_eth_balance() {
    dotenv::dotenv().ok();
    
    // Vitalik Buterin's address
    let wallet_address = "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B".to_string();
    
    let result = get_balance(wallet_address, None).await;
    
    assert!(result.is_ok());
    
    let balance_json = result.unwrap();
    let balance = balance_json.get("balance").and_then(Value::as_str);
    
    assert!(balance.is_some());
    
    // Check if the balance is a valid decimal number
    assert!(balance.unwrap().parse::<f64>().is_ok());
}
