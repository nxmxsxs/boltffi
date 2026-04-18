use boltffi::*;
use demo_bench_macros::benchmark_candidate;

/// Represents a person with a name and age.
#[data]
#[benchmark_candidate(record, uniffi)]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

#[benchmark_candidate(function, uniffi)]
pub fn echo_person(p: Person) -> Person {
    p
}

#[benchmark_candidate(function, uniffi)]
pub fn make_person(name: String, age: u32) -> Person {
    Person { name, age }
}

#[benchmark_candidate(function, uniffi)]
pub fn greet_person(p: Person) -> String {
    format!("Hello, {}! You are {} years old.", p.name, p.age)
}

#[data]
#[benchmark_candidate(record, uniffi)]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub zip: String,
}

#[benchmark_candidate(function, uniffi)]
pub fn echo_address(a: Address) -> Address {
    a
}

#[benchmark_candidate(function, uniffi)]
pub fn format_address(a: Address) -> String {
    format!("{}, {}, {}", a.street, a.city, a.zip)
}
