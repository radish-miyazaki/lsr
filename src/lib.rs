use std::{fs, path::PathBuf};

use clap::Parser;

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

pub fn run() -> MyResult<()> {
    let args = Args::parse();
    let paths = find_files(&args.paths, args.show_hidden)?;
    for path in paths {
        println!("{}", path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::find_files;

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
}
