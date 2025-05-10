use std::process::Command;
use std::path::Path;
use std::env::var;

fn main() {
  #[allow(clippy::single_element_loop)]
  for path in ["build.rs"] {
    println!("cargo::rerun-if-changed={path}");
  }
  let commit_hash = {
    if Path::new("./.git").exists() {
      let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to execute command");

      if output.status.success() {
        let commit_hash_str = String::from_utf8_lossy(&output.stdout);
        commit_hash_str.trim().to_string()
      } else {
        println!("cargo::warning=Git command failed with status: {}", output.status);
        String::from("unknown")
      }
    } else {
      println!("cargo::warning=No .git directory found, skipping git versioning");
      String::from("unknown")
    }
  };

  println!(
    "cargo::rustc-env=VERSION={}({} Profile)-{}({})",
    var("CARGO_PKG_VERSION").unwrap(),
    var("PROFILE").unwrap(),
    commit_hash,
    match var("ACTIONS_ID") {
      Ok(id) => format!("actions/runs/{id}"),
      Err(std::env::VarError::NotPresent) => "Local".to_string(),
      Err(e) => {
        println!("cargo::error={}", e);
        "unknown".to_string()
      }
    }
  );
}
