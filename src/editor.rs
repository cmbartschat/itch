use git2::Error;

pub fn edit_text(_: &str, _: Option<&str>) -> Result<String, Error> {
    // let temp_path = std::path::Path::new("/tmp/example");
    // std::fs::write(temp_path, initial_content).map_err(|e| Error::from_str(&e.to_string()))?;
    // std::process::Command::new("vim").spawn().map?;
    Ok("replaced".to_string())
}
