use crate::results::ComputeError;
use crate::{
    enums::{data_enum::Shape, repr_int::Priority},
    records::{
        blittable::Point,
        mixed::{MixedRecord, MixedRecordParameters, echo_mixed_record, make_mixed_record},
    },
};
use boltffi::*;

/// Adds two numbers asynchronously.
#[export]
#[demo_bench_macros::benchmark_candidate(function, uniffi, wasm_bindgen)]
pub async fn async_add(a: i32, b: i32) -> i32 {
    a + b
}

#[export]
pub async fn async_echo(message: String) -> String {
    format!("Echo: {}", message)
}

#[export]
pub async fn async_double_all(values: Vec<i32>) -> Vec<i32> {
    values.into_iter().map(|v| v * 2).collect()
}

#[export]
pub async fn async_find_positive(values: Vec<i32>) -> Option<i32> {
    values.into_iter().find(|&v| v > 0)
}

#[export]
pub async fn async_concat(strings: Vec<String>) -> String {
    strings.join(", ")
}

#[export]
pub async fn try_compute_async(value: i32) -> Result<i32, ComputeError> {
    crate::results::try_compute(value)
}

#[export]
pub async fn fetch_data(id: i32) -> Result<i32, String> {
    if id > 0 {
        Ok(id * 10)
    } else {
        Err("invalid id".to_string())
    }
}

#[export]
#[demo_bench_macros::benchmark_candidate(function, uniffi)]
pub async fn async_get_numbers(count: i32) -> Vec<i32> {
    (0..count).collect()
}

#[export]
pub async fn async_echo_mixed_record(record: MixedRecord) -> MixedRecord {
    echo_mixed_record(record)
}

#[export]
pub async fn async_make_mixed_record(
    name: String,
    anchor: Point,
    priority: Priority,
    shape: Shape,
    parameters: MixedRecordParameters,
) -> MixedRecord {
    make_mixed_record(name, anchor, priority, shape, parameters)
}
