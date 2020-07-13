use super::*;
use std::io::{Error, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

impl IO for UnixStream {
    type Error = Error;
    fn put(&mut self, data: &[u8]) -> Result<(), Error> {
        self.write_all(data)
    }
    fn get(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.read_exact(data)
    }
}

impl Connection<UnixStream> {
    pub fn connect(path: &Path) -> Result<Self, Error> {
        Self::new(UnixStream::connect(path)?)
    }
}
