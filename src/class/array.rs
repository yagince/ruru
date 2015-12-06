use std::convert::From;

use binding::array;
use types;

use super::object;
use super::traits::RawObject;

pub struct Array {
    value: types::rb_value
}

impl Array {
    pub fn new() -> Self {
        Array {
            value: array::new()
        }
    }

    pub fn at(&self, index: i64) -> object::Object {
        let value = array::entry(self.value(), index);

        object::Object::from(value)
    }

    pub fn push<T: RawObject>(&mut self, item: T) -> &mut Self {
        array::push(self.value(), item.value());

        self
    }
}

impl From<types::rb_value> for Array {
    fn from(value: types::rb_value) -> Self {
        Array {
            value: value
        }
    }
}

impl RawObject for Array {
    fn value(&self) -> types::rb_value {
        self.value
    }
}
