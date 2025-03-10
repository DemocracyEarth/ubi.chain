use jsonrpc_core::{IoHandler, Result, Value};
use jsonrpc_http_server::{ServerBuilder, DomainsValidation};
use jsonrpc_core::futures::future;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub fn start_server() -> Result<jsonrpc_http_server::Server> {
    let mut io = IoHandler::default();
    
    io.add_method("say_hello", |_params| {
        future::ready(Ok(Value::String("hello".into())))
    });
    
    let server = ServerBuilder::new(io)
        // Try different CORS configurations
        // .cors(DomainsValidation::Disabled)
        .cors(DomainsValidation::AllowOnly(vec!["*".into()]))
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .expect("Unable to start RPC server");
        
    Ok(server)
}
