use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let concat_path = Path::new(&out_dir).join("concat.ir");

    let sem_dir = Path::new("src/sem");
    let mut concat_string = String::new();
    visit_dirs(sem_dir, &mut concat_string).unwrap();

    let mut concat_file = File::create(concat_path).unwrap();
    write!(concat_file, "{}", concat_string).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/sem");
}

fn visit_dirs(dir: &Path, result: &mut String) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_dirs(&path, result)?;
            } else if path.is_file() {
                let contents = fs::read_to_string(&path)?;
                result.push_str(&contents);
            }
        }
    }
    Ok(())
}
