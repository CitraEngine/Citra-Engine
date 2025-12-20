use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
};

use citra_engine::logger::Logger;

pub struct N3dsLogger {
    log_file: File,
    write_tag: bool,
}
impl N3dsLogger {
    pub fn new(path: String) -> Result<Self, io::Error> {
        std::fs::create_dir_all(Path::new(&path).parent().unwrap_or(Path::new("")))?;
        Ok(Self {
            log_file: OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?,
            write_tag: true,
        })
    }
}
impl Logger for N3dsLogger {
    fn debug(&mut self, msg: &dyn std::fmt::Display) {
        if self.write_tag {
            let _ = self.log_file.write_all("[Debug]: ".as_bytes());
        }
        let msg = msg.to_string();
        self.write_tag = msg.ends_with("\n");
        let _ = self.log_file.write_all(msg.as_bytes());
        let _ = self.log_file.flush();
    }
    fn info(&mut self, msg: &dyn std::fmt::Display) {
        if self.write_tag {
            let _ = self.log_file.write_all("[Info]: ".as_bytes());
        }
        let msg = msg.to_string();
        self.write_tag = msg.ends_with("\n");
        let _ = self.log_file.write_all(msg.as_bytes());
        let _ = self.log_file.flush();
    }
    fn warn(&mut self, msg: &dyn std::fmt::Display) {
        if self.write_tag {
            let _ = self.log_file.write_all("[Warning]: ".as_bytes());
        }
        let msg = msg.to_string();
        self.write_tag = msg.ends_with("\n");
        let _ = self.log_file.write_all(msg.as_bytes());
        let _ = self.log_file.flush();
    }
    fn error(&mut self, msg: &dyn std::fmt::Display) {
        if self.write_tag {
            let _ = self.log_file.write_all("[Error]: ".as_bytes());
        };
        let msg = msg.to_string();
        self.write_tag = msg.ends_with("\n");
        let _ = self.log_file.write_all(msg.as_bytes());
        let _ = self.log_file.flush();
    }
}
