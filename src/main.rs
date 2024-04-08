#![allow(unused_imports)]

use clap::{Parser, Subcommand};
use std::{ops::RangeInclusive, path::Path, time::Duration};
use tls::tls_connect;
use tokio::runtime;

mod common;
mod connect;
mod server;
mod stream;
mod tls;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Connect {
        host: String,

        #[arg(short, long, value_parser = port_in_range)]
        port: u16,

        #[arg(long, value_parser = valid_path)]
        ca: Option<String>,
    },

    Serve {
        #[arg(default_value = "127.0.0.1")]
        bind_host: String,

        #[arg(short, long, value_parser = port_in_range)]
        port: u16,

        #[arg(long, value_parser = valid_path)]
        ca: Option<String>,

        #[arg(long, value_parser = valid_path)]
        cert: Option<String>,

        #[arg(long, value_parser = valid_path)]
        key: Option<String>,
    },
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{}` is not a valid port number", s))?;

    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "Port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

fn valid_path(s: &str) -> Result<String, String> {
    let path = Path::new(s);

    if path.exists() {
        Ok(s.to_string())
    } else {
        Err(format!("Path does not exist {}", s))
    }
}

fn main() {
    let cli = Cli::parse();

    // custom tokio runtime
    let runtime = runtime::Runtime::new().unwrap();

    match &cli.command {
        Commands::Connect { host, port, ca } => {
            println!("connect to {}:{}", host, port);

            //stream::client().await.unwrap();
            runtime.block_on(async {
                tokio::select! {
                    res = tls::tls_connect(host, port, ca) => {
                        if let Err(e) = res {
                            println!("connect failed: {}", e.to_string());
                        }
                    }

                    //_ = stream::client() => {}
                    //_ = tls_connect(host, port) => {}
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        }
        Commands::Serve {
            bind_host,
            port,
            ca,
            cert,
            key,
        } => {
            println!("bind to {}:{}", bind_host, port);

            //stream::server().await.unwrap();
            runtime.block_on(async {
                tokio::select! {
                    res = tls::tls_listen(bind_host, port, ca, cert.clone().expect("cert is required"), key.clone().expect("key is required")) => {
                        if let Err(e) = res {
                            println!("listen failed: {}", e.to_string());
                        }
                    }

                    //_ = stream::server() => {}
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        }
    }

    // Client Connect Cmd
    // cargo run connect google.com --port 443

    // Serve Cmd
    // cargo run serve localhost --port 2323 --ca .\certs\ca.pem --cert .\certs\server.pem --key .\certs\server-key.pem
    // Connect Cmd for Client to connect to Serve
    // cargo run connect localhost --port 2323 --ca .\certs\ca.pem

    runtime.shutdown_timeout(Duration::from_secs(0));
}
