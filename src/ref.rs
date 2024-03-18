use std::{
    fmt::write,
    fs::{read_dir, DirEntry},
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
};

type FileEntries = Option<Vec<FileEntry>>;

pub enum WalkerError {
    MaxDepthReached,
    FileNotDirectory,
    PathNotFound,
}

impl std::fmt::Display for WalkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Maximum recursion limit reached for walking directories")
    }
}
impl std::fmt::Debug for WalkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Maximum recursion limit reached ")
    }
}

impl std::fmt::Display for FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.path.file_name().unwrap().to_str().unwrap())
    }
}
impl std::fmt::Display for Walker {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.root)
    }
}

#[derive(Debug)]
pub enum VisitStatus {
    Visited,
    Unvisited,
}

#[derive(Debug)]
pub struct FileEntry {
    path: PathBuf,
    children: FileEntries,
    visit_status: VisitStatus,
}

impl FileEntry {
    fn new() -> Self {
        Self {
            path: PathBuf::new(),
            children: None,
            visit_status: VisitStatus::Unvisited,
        }
    }

    fn from_path(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            children: None,
            visit_status: VisitStatus::Unvisited,
        }
    }
    fn from_dir_entry(dir_entry: &DirEntry) -> Self {
        let file = dir_entry.path();
        Self {
            path: file,
            children: None,
            visit_status: VisitStatus::Unvisited,
        }
    }

    fn get_path(&self) -> &Path {
        &self.path
    }

    fn is_dir(&self) -> bool {
        self.path.is_dir()
    }
    fn get_extension(&self) -> Option<String> {
        self.path
            .extension()
            .map(|ext| ext.to_str().unwrap().to_string())
    }

    fn get_size(&self) -> String {
        match self.path.metadata() {
            Ok(metadata) => {
                let mut unit = "B";
                let mut size = metadata.len();
                if size > 1024 * 1024 * 1024 {
                    size = size / (1024 * 1024 * 1024);
                    unit = "GB"
                } else if size > 1024 * 1024 {
                    size = size / (1024 * 1024);
                    unit = "MB"
                } else if size > 1024 {
                    size = size / 1024;
                    unit = "KB"
                }
                format!("{} {}", size, unit)
            }
            Err(_) => String::from("NA"),
        }
    }

    fn visit(&mut self) {
        self.visit_status = VisitStatus::Visited;
    }

    fn add_child(&mut self, dir_entry: FileEntry) {
        match self.children.take() {
            Some(mut dirs) => {
                dirs.push(dir_entry);
                self.children = Some(dirs);
            }
            None => {
                self.children = Some(vec![dir_entry]);
            }
        }
    }

    fn set_children(&mut self, dir_entries: FileEntries) {
        self.children = dir_entries;
    }
}

#[derive(Debug)]

pub struct WalkerOptions {
    is_recursive: bool,
    max_depth: usize,
    show_hidden_files: bool,
}

impl WalkerOptions {
    pub fn set_recursive(mut self, recursive: bool) -> Self {
        self.is_recursive = recursive;
        self
    }
    pub fn set_show_hidden_files(mut self, show: bool) -> Self {
        self.show_hidden_files = show;
        self
    }
}

impl WalkerOptions {
    pub fn new() -> Self {
        Self {
            is_recursive: false,
            max_depth: u8::MAX as usize,
            show_hidden_files: true,
        }
    }
}

#[derive(Debug)]
pub struct Walker {
    options: WalkerOptions,
    root: FileEntry,
}

impl Walker {
    pub fn from_path(root_path: &Path, options: WalkerOptions) -> Result<Self, WalkerError> {
        if !root_path.exists() {
            return Err(WalkerError::PathNotFound);
        }
        let root = FileEntry::from_path(root_path);
        // let unvisited_dir_entries = get_dir_entries(root_path);
        // for dir_entry in unvisited_dir_entries {
        //     let file_entry = FileEntry::from_dir_entry(dir_entry);
        //     root.add_child(file_entry);
        // }

        Ok(Self { root, options })
    }

    pub fn walk_from_root(&mut self) {
        let depth = 0;

        if self.options.is_recursive {
            //take a parent dir
            //walk that directory
            //get all entries
            //self.root=>self.root.children=>
            let mut root = std::mem::replace(&mut self.root, FileEntry::new());
            self.walk_dir_recursive(&mut root, depth);
            self.root = root;
        } else {
            self.root.visit();

            match self.walk_dir(&self.root, depth) {
                Ok(entries) => {
                    self.root.set_children(entries);
                }
                Err(e) => match e {
                    WalkerError::MaxDepthReached => (),
                    WalkerError::FileNotDirectory => (),
                    WalkerError::PathNotFound => (),
                },
            }
        }
    }

    fn walk_dir_recursive(&self, parent: &mut FileEntry, depth: usize) -> Result<(), WalkerError> {
        parent.visit();
        let depth = depth + 1;
        if depth == self.options.max_depth {
            println!("MaxDepth: {:#?}", parent);
            return Err(WalkerError::MaxDepthReached);
        }
        if !parent.is_dir() {
            // println!("Is not a directory: {:#?}", parent);

            return Err(WalkerError::FileNotDirectory);
        }

        let dir_entries = get_dir_entries(parent.get_path());

        if self.options.show_hidden_files {
            // println!("Not a hidden Parent: {:#?}", parent);

            for entry in dir_entries {
                let mut child = FileEntry::from_dir_entry(&entry);
                if entry.path().is_dir() {
                    let _ = self.walk_dir_recursive(&mut child, depth);
                }
                parent.add_child(child);
            }
            Ok(())
        } else {
            // println!("Is a hiddenFile: {:#?}", parent);

            for entry in dir_entries {
                if file_is_hidden(&entry) {
                    continue;
                }
                let mut child = FileEntry::from_dir_entry(&entry);
                if entry.path().is_dir() {
                    let _ = self.walk_dir_recursive(&mut child, depth);
                }
                parent.add_child(child);
            }
            Ok(())
        }
    }

    fn walk_dir(&self, parent: &FileEntry, depth: usize) -> Result<FileEntries, WalkerError> {
        if depth == self.options.max_depth {
            return Err(WalkerError::MaxDepthReached);
        }

        if !parent.is_dir() {
            return Err(WalkerError::FileNotDirectory);
        }

        let dir_entries = get_dir_entries(parent.get_path());

        if self.options.show_hidden_files {
            Ok(dir_entries
                .into_iter()
                .map(|dir_entry| Some(FileEntry::from_dir_entry(&dir_entry)))
                .collect::<FileEntries>())
        } else {
            Ok(dir_entries
                .into_iter()
                .filter(|dir_entry| !file_is_hidden(dir_entry))
                .map(|dir_entry| Some(FileEntry::from_dir_entry(&dir_entry)))
                .collect::<FileEntries>())
        }
    }
    pub fn print(&self) {
        println!("{}:", self.root.path.as_os_str().to_str().unwrap());
        match &self.root.children {
            Some(entries) => {
                for entry in entries.iter() {
                    println!(
                        "[{}]\t{} \t{}",
                        if entry.is_dir() {
                            "DIR".to_string()
                        } else {
                            match entry.get_extension() {
                                Some(ext) => format!("{} FILE", ext),
                                None => String::from("N/A"),
                            }
                        },
                        entry,
                        entry.get_size()
                    );
                    // println!("{}", entry);
                }
            }
            None => {
                println!("THERE ARE NO FILES INSIDE");
            }
        }
    }
}

fn get_dir_entries(path: &Path) -> Vec<DirEntry> {
    let mut dirs = Vec::new();
    match std::fs::read_dir(path) {
        Ok(read_dir) => {
            for entry in read_dir {
                match entry {
                    Ok(f) => {
                        dirs.push(f);
                    }
                    Err(err) => {
                        eprintln!(
                            "ERROR [{}]: Cannot read Directory of path: {}",
                            err,
                            path.to_str().unwrap()
                        )
                    }
                }
            }
        }
        Err(err) => eprintln!(
            "ERROR [{}]: Cannot read Directory of path: {}",
            err,
            path.to_str().unwrap()
        ),
    }
    return dirs;
}

fn file_is_hidden(file: &DirEntry) -> bool {
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;
    let file_attr = match file.metadata() {
        Ok(metadata) => metadata.file_attributes(),
        Err(_) => return false,
    };
    //FILE_ATTRIBUTE_HIDDEN is 0x02 for windows and
    //any number that results in a number greater than zero after bitwise-and with it is hidden
    return file_attr & FILE_ATTRIBUTE_HIDDEN != 0;
}

// read a dir
//get all its file names
//read each file

pub fn test() {
    let mut file = FileEntry::from_path(Path::new("foo.txt"));
    let child_file = FileEntry::new();
    let child_file1 = FileEntry::new();
    let child_file2 = FileEntry::new();
    file.add_child(child_file);
    file.add_child(child_file1);
    file.add_child(child_file2);

    dbg!(file);
}

#[test]
fn add_child() {
    let mut file = FileEntry::from_path(Path::new("foo.txt"));
    let child_file = FileEntry::new();
    file.add_child(child_file);
    let second_child = FileEntry::new();
    let mut second_file = FileEntry {
        path: PathBuf::from("foo.txt"),
        children: Some(vec![second_child]),
        visit_status: VisitStatus::Unvisited,
    };
    dbg!(file);
}
