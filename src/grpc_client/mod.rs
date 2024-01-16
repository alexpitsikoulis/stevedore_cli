use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig, Identity},
    Status, Streaming,
};

use crate::auth::{get_ca_certificate, get_certificate};

mod ca;
pub use ca::CertificateAuthorityClient;

pub mod runner {
    tonic::include_proto!("runner");
}

use runner::{
    runner_client::RunnerClient, QueryJobRequest, QueryJobResponse, StartJobRequest,
    StartJobResponse, StopJobRequest, StopJobResponse, StreamJobRequest, StreamJobResponse,
};

#[derive(Debug)]
pub enum ClientError {
    Transport(tonic::transport::Error),
    Request(Status),
    FileSystem(std::io::Error),
    General(Box<dyn std::error::Error>),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ClientError {}

pub struct Client(pub RunnerClient<Channel>);

impl Client {
    pub async fn new(addr: &'static str, ca_addr: &'static str) -> Result<Self, ClientError> {
        let mut ca = CertificateAuthorityClient::new(ca_addr).await?;

        let ca_certificate = get_ca_certificate(&mut ca)
            .await
            .map_err(ClientError::General)?;
        let (certificate, private_key) = get_certificate(&mut ca)
            .await
            .map_err(ClientError::General)?;

        let client_identity = Identity::from_pem(certificate, private_key);

        let tls_config = ClientTlsConfig::new()
            .identity(client_identity)
            .ca_certificate(Certificate::from_pem(ca_certificate))
            .domain_name("127.0.0.1");

        let channel = Channel::from_static(addr)
            .tls_config(tls_config.clone())
            .map_err(ClientError::Transport)?
            .connect()
            .await
            .map_err(ClientError::Transport)?;

        Ok(Client(RunnerClient::new(channel)))
    }

    pub async fn start(&mut self, req: StartJobRequest) -> Result<StartJobResponse, ClientError> {
        match self.0.start_job(req).await {
            Ok(res) => Ok(res.into_inner()),
            Err(e) => Err(ClientError::Request(e)),
        }
    }

    pub async fn stop(&mut self, req: StopJobRequest) -> Result<StopJobResponse, ClientError> {
        match self.0.stop_job(req).await {
            Ok(res) => Ok(res.into_inner()),
            Err(e) => Err(ClientError::Request(e)),
        }
    }

    pub async fn query(&mut self, req: QueryJobRequest) -> Result<QueryJobResponse, ClientError> {
        match self.0.query_job(req).await {
            Ok(res) => Ok(res.into_inner()),
            Err(e) => Err(ClientError::Request(e)),
        }
    }

    pub async fn stream(
        &mut self,
        req: StreamJobRequest,
    ) -> Result<Streaming<StreamJobResponse>, ClientError> {
        match self.0.stream_job(req).await {
            Ok(res) => Ok(res.into_inner()),
            Err(e) => Err(ClientError::Request(e)),
        }
    }
}
