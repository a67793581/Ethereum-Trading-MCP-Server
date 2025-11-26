# Programming Assignment: Ethereum Trading MCP Server

## Overview

This project is a Model Context Protocol (MCP) server built in Rust that enables AI agents to query balances and execute token swaps on Ethereum.

## Features

- **`get_balance`**: Query ETH and ERC20 token balances.
- **`get_token_price`**: Get current token price in USD using Chainlink price feeds.
- **`swap_tokens`**: Simulate a token swap on Uniswap V2.

## Technical Stack

- **Rust** with **Tokio** for async runtime.
- **ethers-rs** for Ethereum RPC communication.
- **rmcp** for the MCP server framework.
- **rust_decimal** for financial precision.
- **dotenv** for managing environment variables.

## Setup

1.  **Install Rust:**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2.  **Clone the repository:**
    ```bash
    git clone <repository-url>
    cd <repository-name>
    ```

3.  **Set up environment variables:**
    Create a `.env` file in the root of the project and add your Ethereum RPC endpoint and a private key for simulations. You can use a service like [Infura](https://www.infura.io/) or [Alchemy](https://www.alchemy.com/).

    ```env
    # .env
    INFURA_PROJECT_ID=your_infura_project_id
    PRIVATE_KEY=your_wallet_private_key
    ```

4.  **Build the project:**
    ```bash
    cargo build --release
    ```

5.  **Run the server:**
    ```bash
    cargo run --release
    ```
    The server will start on `127.0.0.1:8080`.

## Example MCP Tool Call

Here is an example of calling the `get_balance` tool using `curl`.

**Request:**

```bash
curl -X POST -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "get_balance",
  "params": {
    "wallet_address": "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B"
  },
  "id": 1
}' http://127.0.0.1:8080
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "balance": "..."
  },
  "id": 1
}
```

## Design Decisions

- **ethers-rs vs. alloy:** I chose `ethers-rs` as it is more mature and has extensive documentation, which is beneficial for a project with a tight deadline.
- **Uniswap V2 for Swaps:** I implemented swaps using Uniswap V2 for simplicity. The V2 router provides straightforward functions like `swapExactTokensForTokens`, which is easier to integrate than V3's more complex architecture.
- **Environment Variables for Secrets:** Sensitive information like the Infura project ID and private keys are managed through a `.env` file, which is a standard and secure practice.

## Known Limitations

- **Limited Token Support:** The `get_token_price` and `swap_tokens` tools currently support a hardcoded list of tokens (ETH, DAI, USDC). This could be extended by using a dynamic token list or a discovery mechanism.
- **Uniswap V2 Only:** The swap functionality is limited to Uniswap V2. Adding V3 support would require a more complex implementation to handle concentrated liquidity and path finding.
- **Basic Error Handling:** The error handling is basic. A production-ready server would need more robust error handling and reporting.

## Tests

To run the tests, use the following command:

```bash
cargo test
```

The tests cover the core functionality of the MCP tools. Note that the tests require a valid `INFURA_PROJECT_ID` and `PRIVATE_KEY` in the `.env` file to run.
