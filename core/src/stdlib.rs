use std::io;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

pub fn path_folder(file_name:&str) -> &str {
    let path = Path::new(file_name);
    let root = path.parent().unwrap();

    root.to_str().unwrap()
}

pub fn path_combine(paths:&[&str]) -> PathBuf {
    let path: PathBuf = paths.iter().collect();
    path
}

pub fn read_all(path:&str) -> Result<String, io::Error>
{
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn read_line<R: Read>(of:R) -> io::BufReader<R> {
    io::BufReader::new(of)
}