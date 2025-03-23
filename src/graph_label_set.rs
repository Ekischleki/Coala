use std::rc::Rc;

use crate::label::Label;

pub struct LabelSet {
    labels: Vec<Rc<Label>>
}