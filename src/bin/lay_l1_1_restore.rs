use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "lay-l1.1-restore",
    about = "Shadow L1.1 damaged-surface signal restorer"
)]
struct Args {
    #[arg(long, value_name = "PACKAGE")]
    memory: Option<PathBuf>,

    #[arg(long, value_name = "SOCKET")]
    socket: Option<PathBuf>,

    #[arg(long, default_value_t = 64)]
    limit: usize,

    #[arg(required = true, num_args = 1..)]
    surface: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let surface = args.surface.join(" ");
    let report = if let Some(memory) = args.memory {
        if args.socket.is_some() {
            return Err("choose either --memory or --socket".into());
        }
        lay::nanda_wave::restore_l1_surface(&memory, &surface, args.limit)?
    } else {
        let socket = args
            .socket
            .unwrap_or_else(lay::nanda_wave::default_l11_socket_path);
        let response = lay::nanda_wave::send_l11_service_request(
            &socket,
            &lay::nanda_wave::L1ServiceRequest::Restore {
                surface,
                limit: args.limit,
            },
        )?;
        match response {
            lay::nanda_wave::L1ServiceResponse::Restore { report } => report,
            lay::nanda_wave::L1ServiceResponse::Error { message } => {
                return Err(message.into());
            }
            other => {
                return Err(format!("unexpected L1.1 service response: {other:?}").into());
            }
        }
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
