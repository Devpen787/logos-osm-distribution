use geofabrik::{inspect_pbf, verify_file, GeofabrikIndex};
use std::env;
use std::fs;
use std::process;

fn usage() -> ! {
    eprintln!(
        "Usage:\n  geofabrik-check resolve <index.json> <lp-region-id>\n  geofabrik-check resolve-all <index.json>\n  geofabrik-check verify <snapshot.osm.pbf> <snapshot.osm.pbf.md5>\n  geofabrik-check inspect-pbf <snapshot.osm.pbf>"
    );
    process::exit(64);
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| usage());

    match command.as_str() {
        "resolve" => {
            let index_path = args.next().unwrap_or_else(|| usage());
            let region_id = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage();
            }

            let input = fs::read_to_string(index_path)?;
            let index = GeofabrikIndex::from_json(&input)?;
            let region = index.resolve(&region_id)?;
            println!("{}", serde_json::to_string_pretty(&region)?);
        }
        "resolve-all" => {
            let index_path = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage();
            }

            let input = fs::read_to_string(index_path)?;
            let index = GeofabrikIndex::from_json(&input)?;
            let regions = index.resolve_all()?;
            println!("{}", serde_json::to_string_pretty(&regions)?);
        }
        "verify" => {
            let snapshot_path = args.next().unwrap_or_else(|| usage());
            let manifest_path = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage();
            }

            let manifest = fs::read_to_string(manifest_path)?;
            let report = verify_file(snapshot_path, &manifest)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "inspect-pbf" => {
            let snapshot_path = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage();
            }

            let metadata = inspect_pbf(snapshot_path)?;
            println!("{}", serde_json::to_string_pretty(&metadata)?);
        }
        _ => usage(),
    }

    Ok(())
}
