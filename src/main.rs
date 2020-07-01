use chrono::{offset, DateTime};
use colored::Colorize;
use libc::{
    mode_t, S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR,
};
use std::os::unix::fs::PermissionsExt;
use std::{error, fs, path};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    all: bool,

    #[structopt(short)]
    long: bool,

    // Files to process
    #[structopt(default_value = ".", parse(from_os_str))]
    path: path::PathBuf,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = Opt::from_args();

    if opt.path.is_dir() {
        let entries: Vec<fs::DirEntry> = fs::read_dir(".")?
            .filter_map(|e| e.ok())
            .filter(|e| opt.all || !e.file_name().into_string().unwrap().starts_with("."))
            .collect();

        let (directories, files): (Vec<_>, Vec<_>) =
            entries.iter().partition(|e| e.path().is_dir());

        for file in files {
            print_file(file, opt.long)?;
        }

        for dir in directories {
            print_file(dir, opt.long)?;
        }
    }

    Ok(())
}

fn print_file(entry: &fs::DirEntry, long: bool) -> Result<(), Box<dyn error::Error>> {
    let mut file_name = entry
        .file_name()
        .into_string()
        .or_else(|f| Err(format!("Invalid entry: {:?}", f)))?;
    if entry.path().is_dir() {
        file_name = format!("{}/", file_name.blue());
    }
    let metadata = entry.metadata()?;
    let modified = metadata.modified()?;
    let modified: DateTime<offset::Local> = modified.into();
    let permissions = metadata.permissions();

    if long {
        println!(
            "{} {} {}",
            parse_permissions(permissions.mode()),
            modified.format("%b %e %H:%M"),
            file_name
        );
    } else {
        print!("{}  ", file_name);
    }

    Ok(())
}

fn parse_permissions(mode: u32) -> String {
    let user = triplet(mode as mode_t, S_IRUSR, S_IWUSR, S_IXUSR);
    let group = triplet(mode as mode_t, S_IRGRP, S_IWGRP, S_IXGRP);
    let other = triplet(mode as mode_t, S_IROTH, S_IWOTH, S_IXOTH);
    [user, group, other].join("")
}

fn triplet(mode: mode_t, read: mode_t, write: mode_t, execute: mode_t) -> String {
    match (mode & read, mode & write, mode & execute) {
        (0, 0, 0) => "---",
        (_, 0, 0) => "r--",
        (0, _, 0) => "-w-",
        (0, 0, _) => "--x",
        (_, 0, _) => "r-x",
        (_, _, 0) => "rw-",
        (0, _, _) => "-wx",
        (_, _, _) => "rwx",
    }
    .to_string()
}
