mod crypto;

use clap::{Args, Parser, Subcommand};
use crypto::EncryptedPayload;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "hashit",
    version,
    about = "Deterministic local secret encryption CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt plain text deterministically with a master key
    Encrypt(CryptoArgs),
    /// Decrypt a Hashit payload with a master key
    Decrypt(CryptoArgs),
}

#[derive(Args)]
struct CryptoArgs {
    /// Plain text for encrypt, or payload for decrypt
    input: String,
    /// Master key used for encryption/decryption
    #[arg(short = 'k', long = "master-key", env = "HASHIT_MASTER_KEY")]
    master_key: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Encrypt(args) => EncryptedPayload::encrypt(&args.input, &args.master_key),
        Commands::Decrypt(args) => EncryptedPayload::decrypt(&args.input, &args.master_key),
    };

    match result {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}
