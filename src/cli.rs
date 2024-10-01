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
    #[command(arg_required_else_help = true)]
    Peers {
        /// The torrent file to download
        #[arg()]
        torrent_file: String,
    },
    /// Create a handshake with a peer
    #[command(arg_required_else_help = true)]
    Handshake {
        /// The torrent file to download
        #[arg()]
        torrent_file: String,

        /// The peer to connect to
        #[arg()]
        peer: SocketAddrV4,
    },
    /// Download the piece of a file.
    #[command(arg_required_else_help = true, name = "download_piece")]
    DownloadPiece {
        /// Download output destination
        #[arg(short, long, required = false)]
        output: String,
        /// The torrent file to print information about.
        #[arg()]
        torrent_file: String,
    },
}
