use std::path::PathBuf;

#[derive(Debug)]
struct FileRedirectCfg {
    target_file: PathBuf,
    file_list: Vec<()>
}

struct FileLayout {
    filename: String,
    offset: usize,
    length: usize,
}