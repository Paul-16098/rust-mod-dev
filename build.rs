use std::process::Command;
use std::path::Path;
use std::env::var;

fn main() {
  for path in ["build.rs", ".git/"] {
    println!("cargo::rerun-if-changed={path}");
  }

  if Path::new("./.git").exists() {
    let output = Command::new("git")
      .arg("rev-parse")
      .arg("HEAD")
      .output()
      .expect("Failed to execute command");

    let mut commit_hash = String::from("unknown");
    if output.status.success() {
      let commit_hash_str = String::from_utf8_lossy(&output.stdout);
      commit_hash = commit_hash_str.trim().to_string();
    } else {
      println!("cargo::warning=Git command failed with status: {}", output.status);
    }
    println!(
      "cargo::rustc-env=VERSION={}({} Profile)-{}({})",
      var("CARGO_PKG_VERSION").unwrap(),
      var("PROFILE").unwrap(),
      commit_hash,
      match var("ACTIONS_ID") {
        Ok(id) => format!("actions/runs/{id}"),
        Err(_) => "Local".to_string(),
      }
    );
  }
}
