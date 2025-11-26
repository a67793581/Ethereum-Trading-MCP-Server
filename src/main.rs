use mcp_server::{get_balance, get_token_price, swap_tokens};
use rmcp::{Request, RpcHandler};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let mut rpc_handler = RpcHandler::new();
    rpc_handler.add_tool(get_balance);
    rpc_handler.add_tool(get_token_price);
    rpc_handler.add_tool(swap_tokens);

    let rpc_handler = Arc::new(rpc_handler);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("MCP Server listening on 127.0.0.1:8080");

    loop {
        let (socket, _) = listener.accept().await?;
        let rpc_handler = rpc_handler.clone();

        tokio::spawn(async move {
            let mut stream = tokio::io::BufReader::new(socket);
            let mut buffer = Vec::new();
            if let Ok(_) = tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut buffer).await {
                if let Ok(req) = serde_json::from_slice::<Request>(&buffer) {
                    let res = rpc_handler.handle_request(req).await;
                    if let Ok(res_json) = serde_json::to_vec(&res) {
                        if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut stream.into_inner(), &res_json).await {
                            eprintln!("Failed to write response: {}", e);
                        }
                    }
                }
            }
        });
    }
}
