pub trait Bank {
    type InputType;
    fn load(&mut self, input: Self::InputType);
    fn load_to_reserve(&mut self, input: Self::InputType);
    fn unload_all(&mut self);
}
