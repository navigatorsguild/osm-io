use std::path::PathBuf;

pub fn disk_usage(path: &PathBuf) -> std::io::Result<u64> {
    let mut size: u64 = 0;
    let dir = std::fs::read_dir(path)?;
    for result in dir.into_iter() {
        let entry = result.unwrap();
        if entry.metadata().unwrap().is_dir() {
            size = size + disk_usage(&entry.path())?;
        } else if entry.metadata().unwrap().is_file() {
            size = size + entry.metadata().unwrap().len();
        } else {
            // probably link
        }
    }
    Ok(size)
}

pub fn to_human(size: u64) -> String {
    if size / 0x10000000000_u64 > 0 {
        format!("{:.3}T", size as f64 / 0x10000000000_u64 as f64)
    } else if size / 0x40000000_u64 > 0 {
        format!("{:.3}G", size as f64 / 0x40000000_u64 as f64)
    } else if size / 0x100000_u64 > 0 {
        format!("{:.3}M", size as f64 / 0x100000_u64 as f64)
    } else if size / 0x400_u64 > 0 {
        format!("{:.3}K", size as f64 / 0x400_u64 as f64)
    } else {
        format!("{}", size)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_disk_usage() {
        // println!("{}", to_human(disk_usage(PathBuf::from("./target")).unwrap()));
        // println!("{}", to_human(disk_usage(PathBuf::from("./")).unwrap()));
        // assert_eq!(1, -1)
    }
}
