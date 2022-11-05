use super::tools::IO;

#[derive(Debug)]
pub struct FileHandle {
    pub path: String,
}

impl FileHandle {
    pub fn from(path: String) -> Self {
        Self { path }
    }
}

impl IO for FileHandle {
    fn path(&self) -> &str {
        &self.path
    }
    fn read<'a>(&'a self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.path)
    }
    fn write<'a>(&'a self, content: String) -> Result<(), std::io::Error> {
        std::fs::write(&self.path, content)
    }
}

#[cfg(test)]
mod mocks {
    pub const ERRONEOUS_PATH: &str = "error this path is garbage";
    pub fn mock_read_file(path: &str) -> Result<String, std::io::Error> {
        if path == ERRONEOUS_PATH {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                ERRONEOUS_PATH,
            ))
        } else {
            Ok(String::from(path))
        }
    }
    pub fn mock_write_file(path: &str, content: String) -> Result<(), std::io::Error> {
        if path == ERRONEOUS_PATH {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                ERRONEOUS_PATH,
            ))
        } else {
            Ok(())
        }
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
            println!("{:?}", actual);
            assert!(expected.is_err())
        }
    }

    #[test]
    fn from() {
        let path_and_content = "hello";
        let handle = FileHandle::from(path_and_content.to_string());
        assert_eq!(path_and_content, &handle.path);
    }

    #[test]
    fn exposes_path_getter() {
        let path_and_content = "hello";
        let handle = FileHandle::from(path_and_content.to_string());
        assert_eq!(path_and_content, handle.path());
    }

    #[rstest]
    #[case::should_call_read_file("hello", Ok("hello".to_string()))]
    #[case::should_propagate_error("oh dear", Err(()))]
    fn read(#[case] path: &str, #[case] expected: Result<String, ()>) {
        let temp_dir = TempDir::new().unwrap();
        let child = temp_dir.child(path);
        let path = child.path().to_str().unwrap().to_string();
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
        let path = child.path().to_str().unwrap().to_string();
        let handle = FileHandle::from(path.clone());
        assert_result(expected, handle.write(content.to_string()));
        if let Ok(_) = expected {
            assert_eq!(content, std::fs::read_to_string(path).expect("Bad Test"));
        }
    }
}
