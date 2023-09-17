use core::ops::Deref;

pub struct MutableOption<T> {
    pub value: Option<T>,
}

impl<T> MutableOption<T> {
    pub const fn new(value: Option<T>) -> Self {
        Self { value }
    }

    pub fn get(&self) -> Option<&T> {
        match &self.value {
            Some(inner_value) => Some(inner_value),
            None => None,
        }
    }

    pub fn set(&mut self, value: Option<T>) {
        self.value = value;
    }
}

impl<T> Deref for MutableOption<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
