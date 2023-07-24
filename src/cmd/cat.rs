use clap::ArgMatches;
use libscoop::{operation, QueryOption, Session};
use std::{path::Path, process::Command};

use crate::Result;

pub fn cmd_cat(matches: &ArgMatches, session: &Session) -> Result<()> {
    if let Some(query) = matches.get_one::<String>("package") {
        let queries = vec![query.as_str()];
        let options = vec![QueryOption::Explicit];
        let result = operation::package_query(session, queries, options, false)?;

        match result.len() {
            0 => eprintln!("Could not find package named '{}'.", query),
            1 => {
                let package = &result[0];
                let cat = match is_program_available("bat.exe") {
                    true => "bat.exe",
                    false => "type",
                };
                let config = session.config();
                let cat_args = match cat == "bat.exe" {
                    false => vec![],
                    true => {
                        let cat_style = config.cat_style();
                        vec!["--no-paging", "--style", cat_style, "--language", "json"]
                    }
                };

                let mut child = Command::new("cmd")
                    .arg("/C")
                    .arg(cat)
                    .arg(package.manfest_path())
                    .args(cat_args)
                    .spawn()?;
                child.wait()?;
            }
            _ => {
                eprintln!("Found multiple packages named '{}':\n", query);
                for (idx, pkg) in result.iter().enumerate() {
                    println!(
                        "  {}. {}/{} ({})",
                        idx + 1,
                        pkg.bucket(),
                        pkg.name(),
                        pkg.homepage()
                    );
                }
                eprintln!("\nUse bucket prefix to narrow results.");
            }
        }
    }
    Ok(())
}

/// Check if a given executable is available on the system
fn is_program_available(exe: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for p in path.split(';') {
            let path = Path::new(p).join(exe);
            if std::fs::metadata(path).is_ok() {
                return true;
            }
        }
    }
    false
}
