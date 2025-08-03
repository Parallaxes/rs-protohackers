use std::{env, error::Error};

use p00_smoke_test;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <challenge>", args[0]);
        eprintln!("Example: {} 1", args[0]);
        std::process::exit(1);
    }
    let challenge = &args[1];

    match challenge.as_str() {
        "0" => p00_smoke_test::run().await?,
        _ => {
            eprintln!("Unknown challenge: {}", challenge);
            std::process::exit(1);
        }
    }

    Ok(())
}