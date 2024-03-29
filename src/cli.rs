use clap::{App, Arg, ArgMatches, SubCommand};
use uuid::Uuid;

use crate::grpc_client::runner::{
    QueryJobRequest, StartJobRequest, StopJobRequest, StreamJobRequest,
};

#[derive(Debug)]
pub enum ParseError {
    EmptyArgs,
    GetArgsFailed,
    SubcommandMissing,
    SubcommandNotSupported(String),
    MissingID,
    InvalidID(uuid::Error),
}
pub enum Command {
    Start(StartJobRequest),
    Stop(StopJobRequest),
    Query(QueryJobRequest),
    Stream(StreamJobRequest),
}

pub fn parse_cmd() -> Result<Command, ParseError> {
    let matches = App::new("Stevedore CLI")
        .author("Alex Pitsikoulis")
        .about("CLI to interact with Stevedore's gRPC API")
        .subcommand(
            SubCommand::with_name("start")
                .about("Starts a job with provided command and arguments")
                .arg(
                    Arg::with_name("command")
                        .help("Command to be executed by Stevedore, including its arguments")
                        .multiple(true)
                        .required(true)
                ),
        )
        .subcommand(
            SubCommand::with_name("stop")
                .about("Stops the job with the specified id")
                .args([
                    Arg::with_name("id")
                        .help("UUID of the job to be stopped")
                        .required(true)
                        .index(1),
                    Arg::with_name("gracefully")
                        .short('g')
                        .long("gracefully")
                        .help("Flag to specify whether the process will be stopped gracefully (with SIGTERM rather than SIGKILL)")
                        .takes_value(false),
                ])
        )
        .subcommand(
            SubCommand::with_name("query")
            .about("Queries the status of the job with the specified id")
            .arg(
                Arg::with_name("id")
                    .help("UUID of the job to be queried")
                    .required(true)
                    .index(1)
            )
        )
        .subcommand(SubCommand::with_name("stream")
            .about("Streams the output of the job with specified id")
            .arg(
                Arg::with_name("id")
                    .help("UUID of the job to be streamed")
                    .required(true)
                    .index(1)
            )
        )
        .get_matches();

    match matches.subcommand() {
        Some(("start", args)) => {
            let args = match args.get_many::<String>("command") {
                Some(args) => {
                    if args.len() == 0 {
                        return Err(ParseError::EmptyArgs);
                    }
                    args
                }
                None => return Err(ParseError::GetArgsFailed),
            };
            let mut args: Vec<String> = args.cloned().collect();
            let name = args.remove(0);
            Ok(Command::Start(StartJobRequest { name, args }))
        }
        Some(("stop", args)) => {
            let id = match validate_job_id(args) {
                Ok(id) => id,
                Err(e) => return Err(e),
            };
            Ok(Command::Stop(StopJobRequest {
                job_id: id,
                owner_id: Uuid::new_v4().to_string(),
                gracefully: args.is_present("gracefully"),
            }))
        }
        Some(("query", args)) => {
            let id = match validate_job_id(args) {
                Ok(id) => id,
                Err(e) => return Err(e),
            };
            Ok(Command::Query(QueryJobRequest {
                job_id: id,
                owner_id: Uuid::new_v4().to_string(),
            }))
        }
        Some(("stream", args)) => {
            let id = match validate_job_id(args) {
                Ok(id) => id,
                Err(e) => return Err(e),
            };
            Ok(Command::Stream(StreamJobRequest {
                job_id: id,
                owner_id: Uuid::new_v4().to_string(),
            }))
        }
        Some((other, _)) => Err(ParseError::SubcommandNotSupported(other.into())),
        None => Err(ParseError::SubcommandMissing),
    }
}

fn validate_job_id(args: &ArgMatches) -> Result<String, ParseError> {
    match args.get_one::<String>("id") {
        Some(id) => match Uuid::parse_str(id) {
            Ok(_) => Ok(id.clone()),
            Err(e) => Err(ParseError::InvalidID(e)),
        },
        None => Err(ParseError::MissingID),
    }
}
