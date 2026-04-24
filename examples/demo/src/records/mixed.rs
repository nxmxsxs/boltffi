use boltffi::*;

use crate::enums::data_enum::Shape;
use crate::enums::repr_int::Priority;
use crate::records::blittable::Point;

#[data]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MixedRecordParameters {
    pub tags: Vec<String>,
    pub checkpoints: Vec<Point>,
    pub fallback_anchor: Option<Point>,
    pub max_retries: u32,
    pub preview_only: bool,
}

#[data]
#[derive(Clone, Debug, PartialEq)]
pub struct MixedRecord {
    pub name: String,
    pub anchor: Point,
    pub priority: Priority,
    pub shape: Shape,
    pub parameters: MixedRecordParameters,
}

impl MixedRecord {
    pub fn from_parts(
        name: String,
        anchor: Point,
        priority: Priority,
        shape: Shape,
        parameters: MixedRecordParameters,
    ) -> Self {
        Self {
            name,
            anchor,
            priority,
            shape,
            parameters,
        }
    }
}

#[export]
pub fn echo_mixed_record(record: MixedRecord) -> MixedRecord {
    record
}

#[export]
pub fn make_mixed_record(
    name: String,
    anchor: Point,
    priority: Priority,
    shape: Shape,
    parameters: MixedRecordParameters,
) -> MixedRecord {
    MixedRecord::from_parts(name, anchor, priority, shape, parameters)
}
