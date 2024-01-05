use runner::runner_client::RunnerClient;
use runner::{QueryJobRequest, StartJobRequest, StopJobRequest, StreamJobRequest};
use tonic::transport::Channel;
use tonic::{Status, Streaming};

use self::runner::{QueryJobResponse, StartJobResponse, StopJobResponse, StreamJobResponse};

pub mod runner {
    tonic::include_proto!("runner");
}

#[derive(Debug)]
pub enum ClientError {
    TransportErr(tonic::transport::Error),
    RequestErr(Status),
}

pub struct Client(RunnerClient<Channel>);

impl Client {
    pub async fn new(addr: String) -> Result<Self, ClientError> {
        match RunnerClient::connect(addr).await {
            Ok(client) => Ok(Client(client)),
            Err(e) => Err(ClientError::TransportErr(e)),
        }
    }

    pub async fn start(&mut self, req: StartJobRequest) -> Result<StartJobResponse, ClientError> {
        match self.0.start_job(req).await {
            Ok(res) => Ok(res.into_inner()),
            Err(e) => Err(ClientError::RequestErr(e)),
        }
    }

    pub async fn stop(&mut self, req: StopJobRequest) -> Result<StopJobResponse, ClientError> {
        match self.0.stop_job(req).await {
            Ok(res) => Ok(res.into_inner()),
            Err(e) => Err(ClientError::RequestErr(e)),
        }
    }

    pub async fn query(&mut self, req: QueryJobRequest) -> Result<QueryJobResponse, ClientError> {
        match self.0.query_job(req).await {
            Ok(res) => Ok(res.into_inner()),
            Err(e) => Err(ClientError::RequestErr(e)),
        }
    }

    pub async fn stream(
        &mut self,
        req: StreamJobRequest,
    ) -> Result<Streaming<StreamJobResponse>, ClientError> {
        match self.0.stream_job(req).await {
            Ok(res) => Ok(res.into_inner()),
            Err(e) => Err(ClientError::RequestErr(e)),
        }
    }
}
