

use crate::{Dep, Message};

pub struct Listener<'a> {
    dep: &'a dyn Dep,
}

impl<'a> Iterator for Listener<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub fn create_listener<'a>(/* TODO: bus type as parameter */ dep: &'a dyn Dep,) -> Listener<'a> {
    Listener { dep }
}
