use std::{fs, os::unix::fs::MetadataExt, path::PathBuf};

use chrono::{DateTime, Local};
use clap::Parser;
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(
    name = "lsr",
    version = "0.1.0",
    author = "Radish-Miyazaki <y.hidaka.kobe@gmail.com>",
    about = "Rust ls"
)]
pub struct Args {
    #[arg(help = "Files and/or directories", default_value = ".")]
    paths: Vec<String>,
    #[arg(help = "Long listing", short, long)]
    long: bool,
    #[arg(help = "Show all files", short = 'a', long = "all")]
    show_hidden: bool,
}

fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut results = vec![];

    for path in paths {
        match fs::metadata(path) {
            Err(e) => {
                eprintln!("{}: {}", path, e);
                continue;
            }
            Ok(m) => {
                if m.is_file() {
                    results.push(PathBuf::from(path));
                } else {
                    fs::read_dir(path)?.for_each(|e| {
                        if let Ok(e) = e {
                            if show_hidden || !e.file_name().to_string_lossy().starts_with('.') {
                                results.push(e.path());
                            }
                        }
                    })
                }
            }
        }
    }

    Ok(results)
}

fn format_mode(mode: u32) -> String {
    let mut s = String::with_capacity(9);
    let rwx = ['r', 'w', 'x'];
    for i in (0..=2).rev() {
        for (j, p) in rwx.iter().enumerate() {
            if mode & (1 << (3 * i + (2 - j))) == 0 {
                s.push('-');
            } else {
                s.push(*p);
            }
        }
    }

    s
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    let fmt = "{:<}{:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let metadata = path.metadata()?;
        let uid = metadata.uid();
        let gid = metadata.gid();
        let user_name = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or_else(|| uid.to_string());
        let group_name = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().to_string())
            .unwrap_or_else(|| gid.to_string());
        let updated_at: DateTime<Local> = DateTime::from(metadata.modified()?);

        table.add_row(
            Row::new()
                .with_cell(if path.is_dir() { "d" } else { "-" }) // file type (d or -)
                .with_cell(format_mode(metadata.mode())) // permissions
                .with_cell(metadata.nlink()) // link count
                .with_cell(user_name) // user name
                .with_cell(group_name) // group name
                .with_cell(if path.is_file() {
                    metadata.size().to_string()
                } else {
                    "".to_string()
                }) // file size
                .with_cell(updated_at.format("%H:%M")) // updated at
                .with_cell(path.display()), // path
        );
    }

    Ok(format!("{}", table))
}

pub fn run() -> MyResult<()> {
    let args = Args::parse();
    let paths = find_files(&args.paths, args.show_hidden)?;

    if args.long {
        let output = format_output(&paths)?;
        for line in output.lines() {
            println!("{}", line);
        }
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::format_output;

    use super::{find_files, format_mode};

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(!parts.is_empty() && parts.len() <= 10);

        let perms = parts.first().unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_find_files() {
        // ディレクトリにある隠しエントリ以外のエントリを返す
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|e| e.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt"
            ]
        );

        // ファイルを直接指定した場合は、隠しファイルであっても返す
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|e| e.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // ファイルとディレクトリのパスをそれぞれ与えた場合
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|e| e.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|e| e.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt"
            ]
        )
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o644), "rw-r--r--");
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);
        let empty_line = lines.remove(0);
        long_match(empty_line, "tests/inputs/empty.txt", "-rw-r--r--", None);
    }
}
