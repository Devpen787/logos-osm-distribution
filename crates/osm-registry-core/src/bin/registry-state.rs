use osm_registry_core::Registry;
use std::env;
use std::fs;
use std::process;

fn usage() -> ! {
    eprintln!(
        "Usage:\n  registry-state <state.bin> decode\n  registry-state <state.bin> region <region>\n  registry-state <state.bin> parent <parent>\n  registry-state <state.bin> cid <cid>\n  registry-state <state.bin> latest <region>"
    );
    process::exit(2);
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let path = args.next().unwrap_or_else(|| usage());
    let command = args.next().unwrap_or_else(|| usage());

    let bytes = fs::read(path)?;
    let registry: Registry = borsh::from_slice(&bytes)?;

    match command.as_str() {
        "decode" => {
            if args.next().is_some() {
                usage();
            }
            println!("{}", serde_json::to_string_pretty(&registry)?);
        }
        "region" => {
            let region = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage();
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&registry.by_region(&region))?
            );
        }
        "parent" => {
            let parent = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage();
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&registry.by_parent(&parent))?
            );
        }
        "cid" => {
            let cid = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage();
            }
            println!("{}", serde_json::to_string_pretty(&registry.by_cid(&cid))?);
        }
        "latest" => {
            let region = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage();
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&registry.latest_by_region(&region))?
            );
        }
        _ => usage(),
    }

    Ok(())
}
