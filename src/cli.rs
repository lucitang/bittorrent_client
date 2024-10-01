use clap::{Parser, Subcommand};
use std::net::SocketAddrV4;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "bittorrent-starter-rust")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub subcmd: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Decode a bencoded value
    /// ex: `cargo run decode 5:hello`
    #[command(arg_required_else_help = true)]
    Decode {
        /// The bencoded value to decode
        #[arg()]
        encoded_value: String,
    },
    /// Print information about a torrent file
    /// ex: `cargo run info sample.torrent`
    #[command(arg_required_else_help = true)]
    Info {
        /// The torrent file to print information about.
        #[arg()]
        torrent_file: String,
    },

    /// Discover peers to download a torrent file from.
    Peers {
        /// The torrent file to download
        #[arg()]
        torrent_file: String,
    },
    Handshake {
        /// The torrent file to download
        #[arg()]
        torrent_file: String,

        /// The peer to connect to
        #[arg()]
        peer: SocketAddrV4,
    },
}
