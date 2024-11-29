use std::path::Path;

pub fn get_drives() -> Vec<String> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .filter_map(|c| {
                let drive = format!("{}:\\", c as char);
                Path::new(&drive)
                    .metadata()
                    .ok()
                    .filter(|m| m.is_dir())
                    .map(|_| drive)
            })
            .collect()
    }

    #[cfg(not(windows))]
    {
        vec!["/".to_string()]
    }
}