use std::rc::Rc;

use crate::label::Label;
#[derive(Default, Debug)]

pub struct LabelSet {
    labels: Vec<Rc<Label>>
}