#[cfg(test)]
mod tests {
    use crate::helpers::Fp;
    use crate::library::Library;
    use crate::settings::LibraryMode;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    fn setup_library() -> (Library, PathBuf, TempDir) {
        let dir = tempdir().expect("tempdir failed");
        let home = dir.path().canonicalize().expect("canonicalize failed");

        fs::create_dir_all(home.join("metadata")).expect("create_dir_all metadata failed");
        fs::File::create(home.join(".fat32_epoch")).expect("create fat32_epoch failed");

        let library = Library::new(&home, LibraryMode::Filesystem).expect("Library::new failed");
        (library, home, dir)
    }

    #[test]
    fn test_rename_file() {
        let (mut library, home, _dir) = setup_library();
        let file_name = "test.txt";
        let file_path = home.join(file_name);

        fs::write(&file_path, "hello").expect("write file failed");

        let rel_path = PathBuf::from(file_name);
        library.paths.insert(rel_path.clone(), Fp(12345));

        // Use a fully canonicalized path to guarantee we are talking about the exact same file
        let abs_src = file_path.canonicalize().expect("file path should exist");

        // Use the relative path for renaming
        match library.rename(file_name, "new.txt") {
            Ok(_) => (),
            Err(e) => {
                panic!(
                    "Rename failed for {:?} (exists: {}). Error context: {:?}",
                    abs_src,
                    abs_src.exists(),
                    e
                );
            }
        }

        assert!(
            !home.join("test.txt").exists(),
            "Source file should be gone"
        );
        assert!(
            home.join("new.txt").exists(),
            "Destination file should exist"
        );
        assert!(library.paths.contains_key(&PathBuf::from("new.txt")));
        assert!(!library.paths.contains_key(&rel_path));
    }
}
