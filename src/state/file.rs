#[cfg(test)]
use mockall::automock;
#[cfg(test)]
use mocks::mock_read_file as read_file;
#[cfg(test)]
use mocks::mock_write_file as write_file;

#[cfg(not(test))]
use std::fs::read_to_string as read_file;
#[cfg(not(test))]
use std::fs::write as write_file;

#[derive(Debug)]
pub struct FileHandle {
    pub path: String,
}

#[cfg_attr(test, automock())]
impl FileHandle {
    pub fn from(path: String) -> Self {
        FileHandle { path }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn read<'a>(&'a self) -> Result<String, std::io::Error> {
        read_file(&self.path)
    }
    pub fn write<'a>(&'a self, content: String) -> Result<(), std::io::Error> {
        write_file(&self.path, content)
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

    #[test]
    fn read_should_call_read_file() {
        let path_and_content = "hello";
        let handle = FileHandle::from(path_and_content.to_string());
        let actual = handle.read().unwrap();
        assert_eq!(path_and_content, &actual);
    }

    #[test]
    fn read_should_propagate_error() {
        let path_and_content = mocks::ERRONEOUS_PATH;
        let handle = FileHandle::from(path_and_content.to_string());
        assert!(handle.read().is_err());
    }

    #[test]
    fn write_should_call_write_file() {
        let (path, content) = ("hello", "world");
        let handle = FileHandle::from(path.to_string());
        let actual = handle.write(content.to_string()).unwrap();
        assert_eq!((), actual);
    }

    #[test]
    fn write_should_propagate_error() {
        let (path, content) = (mocks::ERRONEOUS_PATH, "");
        let handle = FileHandle::from(path.to_string());
        let actual = handle.write(content.to_string());
        assert!(handle.read().is_err());
    }
}
