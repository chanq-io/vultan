use super::tools::IO;
use anyhow::Result;

#[derive(Debug)]
pub struct FileHandle {
    pub path: std::path::PathBuf,
}

impl FileHandle {
    pub fn from(path: std::path::PathBuf) -> Self {
        Self { path }
    }
}

impl IO for FileHandle {
    fn path(&self) -> &str {
        &self.path.to_str().unwrap_or("unknown")
    }
    fn read<'a>(&'a self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.path)
    }
    fn write<'a>(&'a self, content: String) -> Result<(), std::io::Error> {
        std::fs::write(&self.path, content)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use assert_fs::{fixture::TempDir, prelude::*};
    use rstest::*;

    fn assert_result<T: std::fmt::Debug + PartialEq, E1: std::fmt::Debug, E2: std::fmt::Debug>(
        expected: Result<T, E1>,
        actual: Result<T, E2>,
    ) {
        if let Ok(actual) = actual {
            assert_eq!(expected.expect("BAD TEST"), actual);
        } else {
            assert!(expected.is_err())
        }
    }

    #[test]
    fn from() {
        let path = std::path::PathBuf::from("hello");
        let handle = FileHandle::from(path.clone());
        assert_eq!(path, handle.path);
    }

    #[test]
    fn exposes_path_getter() {
        let path_str = "hello";
        let path = std::path::PathBuf::from(path_str.clone());
        let handle = FileHandle::from(path);
        assert_eq!(path_str, handle.path());
    }

    #[rstest]
    #[case::should_call_read_file("hello", Ok("hello".to_string()))]
    #[case::should_propagate_error("oh dear", Err(()))]
    fn read(#[case] path: &str, #[case] expected: Result<String, ()>) {
        let temp_dir = TempDir::new().unwrap();
        let child = temp_dir.child(path);
        let path = child.path().to_path_buf();
        match expected.clone() {
            Ok(expected) => {
                child.write_str(expected.as_str()).expect("Bad Test");
            }
            _ => {}
        }
        let handle = FileHandle::from(path);
        assert_result(expected, handle.read());
        temp_dir.close().unwrap();
    }

    #[rstest]
    #[case::should_call_write_file("hello", "world", Ok(()))]
    #[case::should_propagate_error("hello///", "", Err(()))]
    fn write(#[case] path: &str, #[case] content: &str, #[case] expected: Result<(), ()>) {
        let temp_dir = TempDir::new().unwrap();
        let child = temp_dir.child(path);
        let path = child.path().to_path_buf();
        let handle = FileHandle::from(path.clone());
        assert_result(expected, handle.write(content.to_string()));
        if let Ok(_) = expected {
            assert_eq!(content, std::fs::read_to_string(path).expect("Bad Test"));
        }
    }
}
