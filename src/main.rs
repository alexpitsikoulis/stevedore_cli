mod auth;
mod cli;
mod grpc_client;

use cli::{Command, ParseError};
use grpc_client::{Client, ClientError};

#[derive(Debug)]
enum Error {
    ParseFailed(ParseError),
    _ClientFailure(ClientError),
    _General(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new(
        "https://127.0.0.1:50051",
        "http://127.0.0.1:50052",
    )
    .await?;
    let cmd = cli::parse_cmd().map_err(Error::ParseFailed)?;

    match cmd {
        Command::Start(args) => {
            match client.start(args).await {
                Ok(res) => {
                    println!("{}", res.job_id);
                }
                Err(e) => {
                    eprintln!("FAILED to start job: {:?}", e);
                }
            };
        }
        Command::Stop(args) => {
            match client.stop(args.clone()).await {
                Ok(_) => {
                    println!("job stopped successfully");
                }
                Err(e) => {
                    eprintln!(
                        "FAILED to stop job {}, ensure that the process is stopped: {:?}",
                        args.job_id, e
                    )
                }
            };
        }
        Command::Query(args) => match client.query(args.clone()).await {
            Ok(res) => {
                println!("{:?}", res);
            }
            Err(e) => {
                eprintln!("FAILED to query status of job {}: {:?}", args.job_id, e);
            }
        },
        Command::Stream(args) => match client.stream(args.clone()).await {
            Ok(mut stream) => loop {
                match stream.message().await {
                    Ok(message) => {
                        match message {
                            Some(message) => {
                                let output = match String::from_utf8(message.output) {
                                    Ok(output) => output,
                                    Err(e) => {
                                        eprintln!(
                                            "failed to cast stream output to string: {:?}",
                                            e
                                        );
                                        break;
                                    }
                                };
                                print!("{}", output);
                            }
                            None => {
                                break;
                            }
                        };
                    }
                    Err(e) => {
                        eprintln!("failed to receive message from output stream: {:?}", e);
                    }
                };
            },
            Err(e) => {
                eprintln!("FAILED to stream output for job {}: {:?}", args.job_id, e)
            }
        },
    };
    Ok(())
}
