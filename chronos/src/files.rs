/// Verify that directory structure exists
/// 
/// This program check that file sturcture exist which is requires for the program.
/// If some directory does not exist, it will try to create it.
/// 
/// # Return values
/// 
/// Return with `Result<(), String>. In case of Err, the parameter is the name of directory which had problem.
pub fn check_and_build_dirs(root: &str) -> Result<(), String> {
    if let Err(_) = create_dir_if_not_exist(root) {
        return Err(String::from(root));
    }

    if let Err(_) = create_dir_if_not_exist(format!("{}/all_timers", root).as_str()) {
        return Err(format!("{}/all_timers", root));
    }

    if let Err(_) = create_dir_if_not_exist(format!("{}/active_timers", root).as_str()) {
        return Err(format!("{}/active_timers", root));
    }

    if let Err(_) = create_dir_if_not_exist(format!("{}/logs", root).as_str()) {
        return Err(format!("{}/log", root));
    }

    return Ok(());
}

/// If specified directory does not exist, then try to create it.
fn create_dir_if_not_exist(path: &str) -> Result<(), ()> {
    if !std::path::Path::new(path).is_dir() {
        if let Err(_) = std::fs::create_dir_all(path) {
            return Err(());
        }
    }
    return Ok(());
}