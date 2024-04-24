use std::{path::Path, process::Command};

use anyhow::Result;

pub fn parse_file(path: &str) -> Result<String> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(anyhow::anyhow!("File not found"));
    }
    let extension = path_obj.extension().unwrap().to_str().unwrap();
    match extension {
        "docx" => {
            todo!();
        }
        "pdf" => {
            let output = Command::new("pdftotext").arg(path).arg("-").output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        "txt" | "md" => {
            let output = std::fs::read_to_string(path)?;
            Ok(output)
        }
        _ => Err(anyhow::anyhow!("Unsupported file format")),
    }
}
