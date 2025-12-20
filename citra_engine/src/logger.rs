use std::fmt::Display;

pub trait Logger {
    fn debug(&mut self, msg: &dyn Display);
    fn info(&mut self, msg: &dyn Display);
    fn warn(&mut self, msg: &dyn Display);
    fn error(&mut self, msg: &dyn Display);
}
