use std::ffi::{OsStr, OsString};

use crate::error::{Maybe, fail, inner_fail};

pub fn edit_temp_text(initial_content: &str, extension: Option<&OsStr>) -> Maybe<String> {
    let mut temp_path = std::env::temp_dir();
    let mut filename: OsString = "itch_edit_buffer.".into();
    if let Some(ext) = extension {
        filename.push(ext);
    } else {
        filename.push("txt");
    }

    temp_path.push(filename);

    std::fs::write(&temp_path, initial_content).map_err(|e| inner_fail(&e.to_string()))?;

    let editor_command = match std::env::var("EDITOR") {
        Ok(v) => std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("{} {}", v, temp_path.to_string_lossy()))
            .spawn(),
        Err(std::env::VarError::NotPresent) => {
            std::process::Command::new("vim").arg(&temp_path).spawn()
        }
        Err(_) => {
            return fail("Unexpected error reading EDITOR variable");
        }
    };

    let mut process = editor_command.map_err(|_| inner_fail("Failed to start edit."))?;

    let status = process
        .wait()
        .map_err(|_| inner_fail("Failed to complete edit."))?;

    if !status.success() {
        return fail("Edit sesssion exited with failure.");
    }

    let res = std::fs::read_to_string(&temp_path)
        .map_err(|_| inner_fail("Failed to read file after edit."))?;

    std::fs::remove_file(&temp_path)
        .map_err(|_| inner_fail("Failed to cleanup file after edit."))?;

    Ok(res)
}
