
mod echo;

use echo::{EchoResponse, EchoRequest};
use std::net::SocketAddr;
use tonic::{Request, Response, Status};

type EchoResult<T> = Result<Response<T>, Status>;

#[derive(Debug)]
pub struct EchoServer {
    addr: SocketAddr,
}

#[tonic::async_trait]
impl echo::echo_server::Echo for EchoServer {
    async fn unary_echo(&self, request: Request<EchoRequest>) -> EchoResult<EchoResponse> {
        let message = format!("{} (from {})", request.into_inner().message, self.addr);

        Ok(Response::new(EchoResponse { message }))
    }
}

#[cfg(test)]
mod test {
    use tonic::{transport::{Server, Channel}, Request};
    use crate::{EchoServer, echo::{self, EchoRequest, echo_client::EchoClient}};

    async fn start_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = address.parse()?;
        let server = EchoServer { addr };
        let serve = Server::builder()
            .add_service(echo::echo_server::EchoServer::new(server))
            .serve(addr);
    
        tokio::spawn(async move {
            if let Err(e) = serve.await {
                eprintln!("Error = {:?}", e);
            }
        });
        Ok(())
    }

    async fn start_servers(http_addresses: &[&str]) {
        for http_address in http_addresses {
            start_server(http_address.strip_prefix("http://").unwrap()).await.unwrap();
        }
    }

    fn make_request() -> Request<EchoRequest> {
        tonic::Request::new(EchoRequest {
            message: "hello".into(),
        })
    }

    #[tokio::test]
    async fn test_with_balance_channel_several_endpoints() {
        let http_addresses = ["http://[::1]:50052","http://[::1]:50053", "http://[::1]:50054", "http://[::1]:50055",
        "http://[::1]:50056", "http://[::1]:50057", "http://[::1]:50058"];
        let endpoints = http_addresses
            .iter()
            .map(|address| Channel::from_static(address));
        let channel = Channel::balance_list(endpoints);
        let mut client = EchoClient::new(channel);

        // No servers started. We should receive an error.
        let mut response_result = client.unary_echo(make_request()).await;
        assert!(response_result.is_err());

        // Start servers.
        start_servers(&http_addresses).await;
    
        // Servers started. We should receive the response and we receive many errors instead.
        let mut error_count = 0;
        response_result = client.unary_echo(make_request()).await;
        while response_result.is_err() {
            error_count += 1;
            response_result = client.unary_echo(make_request()).await;
        }

        assert_eq!(0, error_count, "Error count should be at 0. But in this test it will generall fall between 3 and 6.");
    }

    #[tokio::test]
    async fn test_with_balance_channel_one_endpoint() {
        let http_addresses = ["http://[::1]:50051"];
        let endpoints = http_addresses
            .iter()
            .map(|address| Channel::from_static(address));
        let channel = Channel::balance_list(endpoints);
        let mut client = EchoClient::new(channel);

        // No servers started. We should receive an error.
        let response_result = client.unary_echo(make_request()).await;
        assert!(response_result.is_err());

        // Start servers.
        start_servers(&http_addresses).await;
    
        // Servers started. We should receive the response.
        let response_result = client.unary_echo(make_request()).await;
        assert!(response_result.is_ok());
    }
    
}
