use std::path::Path;

pub fn check_and_create_dir(dir_path: Option<&String>) -> i32 {
    match dir_path {
        Some(dir) => {
            let dir = Path::new(dir);
            if !dir.exists() || (dir.exists() && dir.is_file()) {
                println!("Directory '{}' does not exist, create it...", dir.display());
                if let Err(e) = std::fs::create_dir(dir) {
                    eprintln!("Failed to create '{}' directory: {}", dir.display(), e);
                    return 4;
                }
                println!("Directory '{}' is created!", dir.display());    
            }
        },
        None => {
            eprintln!("Property 'timer.all_dir' is not specified in config");
            return 4;
        }
    }

    return 0;
}