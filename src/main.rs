//tree [path?]

use std::{fs, os::windows::fs::MetadataExt, path::PathBuf, str::FromStr};

use anyhow::{Context, Result};

#[derive(Debug)]
enum TreeEntry {
    DirNode(Directory),
    FileNode(File),
    SymlinkNode(Symlink),
}

#[derive(Debug)]
struct File {
    name: String,
    metadata: Option<fs::Metadata>,
}

#[derive(Debug)]
struct Directory {
    name: String,
    subdirectories: Vec<TreeEntry>,
}

#[derive(Debug)]
struct Symlink {
    name: String,
    target: String,
    metadata: Option<fs::Metadata>,
}

fn walk_dir(path: &PathBuf) -> Result<Directory> {
    let dir_iter = std::fs::read_dir(path).context(format!("unable to read dir: {path:#?}"))?;

    let mut sub_dirs: Vec<TreeEntry> = Vec::new();

    for dir_entry in dir_iter {
        let node: TreeEntry;
        match dir_entry {
            Ok(entry) => {
                // println!("entry=> {entry:#?}");

                node = match entry {
                    file_entry if entry.path().is_file() => {
                        //do file things
                        if file_entry.file_name().to_str().unwrap().starts_with(".") {
                            continue;
                        }
                        if let Ok(metadata) = file_entry.metadata() {
                            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;
                            let file_attr = metadata.file_attributes();

                            //FILE_ATTRIBUTE_HIDDEN is 0x02 for windows and
                            //any number that results in a number greater than zero after bitwise-and with it is hidden
                            if file_attr & FILE_ATTRIBUTE_HIDDEN != 0 {
                                //file is hidden

                                continue;
                            }
                        }
                        TreeEntry::FileNode(File {
                            name: file_entry.file_name().to_str().unwrap().to_string(),
                            metadata: file_entry.metadata().ok(),
                        })
                    }
                    sym_entry if entry.path().is_symlink() => {
                        //proceed with symbolic linky things
                        TreeEntry::SymlinkNode(Symlink {
                            name: sym_entry.file_name().to_str().unwrap().to_string(),
                            target: fs::read_link(sym_entry.path())?.to_string_lossy().into(),
                            metadata: sym_entry.metadata().ok(),
                        })
                    }
                    dir_entry if entry.path().is_dir() => {
                        //do file things
                        if dir_entry.file_name().to_str().unwrap().starts_with(".") {
                            // println!(
                            //     "skipping dir cuz startswith '.'=> {}",
                            //     dir_entry.file_name().to_str().unwrap()
                            // );
                            continue;
                        }
                        if let Ok(metadata) = dir_entry.metadata() {
                            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;
                            let file_attr = metadata.file_attributes();

                            //FILE_ATTRIBUTE_HIDDEN is 0x02 for windows and
                            //any number that results in a number greater than zero after bitwise-and with it is hidden
                            if file_attr & FILE_ATTRIBUTE_HIDDEN != 0 {
                                //file is hidden

                                continue;
                            }
                        }
                        //proceed with directory recursion
                        TreeEntry::DirNode(walk_dir(&dir_entry.path())?)
                    }

                    _ => unreachable!(),
                }
            }
            Err(_) => {
                continue;
            }
        }
        sub_dirs.push(node);
    }

    Ok(Directory {
        name: path.file_name().unwrap().to_str().unwrap().into(),
        subdirectories: sub_dirs,
    })
}

fn print_usage() {
    println!("tree [path]\n[param]=> parameter 'param' is optional;path is optional");
}

fn print_tree(path: &PathBuf, tree: &Directory) {
    const PIPE: &str = "\u{2502}\u{00A0}\u{00A0}"; // │
    const TEE_PIPE: &str = "\u{251c}\u{2500}\u{2500} "; // ├──
    const SPACES: &str = "\u{00A0}\u{00A0} "; // └─
    const L_PIPE: &str = "\u{2514}\u{2500} "; // └─

    println!("{}", path.to_string_lossy());
    let (f, d) = visit(tree, "");
    println!("{} files, {} directories", f, d);

    fn visit(dir: &Directory, pre: &str) -> (usize, usize) {
        let mut dir_count = 1;
        let mut file_count = 0;

        let mut subdir_count = dir.subdirectories.len();

        for entry in dir.subdirectories.iter() {
            subdir_count -= 1;
            let prefix = pre;
            let connector = if subdir_count == 0 { L_PIPE } else { TEE_PIPE };
            match entry {
                TreeEntry::FileNode(file) => {
                    file_count += 1;
                    println!("{}{}{}", prefix, connector, file.name,);
                }
                TreeEntry::SymlinkNode(_) => {
                    file_count += 1;
                }
                TreeEntry::DirNode(dir_entry) => {
                    println!("{}{}{}", prefix, connector, dir_entry.name);
                    let next_prefix = format!(
                        "{}{}",
                        prefix,
                        if subdir_count == 0 { SPACES } else { PIPE }
                    );

                    let (f, d) = visit(dir_entry, &next_prefix);
                    file_count += f;
                    dir_count += d;
                }
            }
        }

        (file_count, dir_count)
    }
}

fn main() -> Result<()> {
    let args = std::env::args();

    let path = match args.skip(1).next() {
        Some(p) => PathBuf::from_str(&p).context(format!("Path cannot be created from {p}"))?,
        None => std::env::current_dir().context(format!("Cannot create starting path"))?,
    };
    let tree = walk_dir(&path)?;
    print_tree(&path, &tree);
    Ok(())
}
