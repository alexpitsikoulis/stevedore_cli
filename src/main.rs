mod cli;
mod client;
use cli::Command;
use client::Client;

#[tokio::main]
async fn main() {
    let cmd = match cli::parse_cmd() {
        Ok(cmd) => cmd,
        Err(e) => {
            eprint!("failed to parse args: {:?}", e);
            return;
        }
    };

    let mut client = match Client::new("http://[::1]:50051".into()).await {
        Ok(client) => client,
        Err(e) => {
            eprint!("failed to initialize gRPC client: {:?}", e);
            return;
        }
    };

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
                Ok(res) => {
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
                eprintln!("FAILED to query status of job {}", args.job_id);
            }
        },
        Command::Stream(args) => match client.stream(args.clone()).await {
            Ok(stream) => {
                todo!()
            }
            Err(e) => {
                eprintln!("FAILED to stream output for job {}: {:?}", args.job_id, e)
            }
        },
    };
}
