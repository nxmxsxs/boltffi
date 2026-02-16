# BoltFFI Benchmarks

Performance comparison across platforms:
- **Swift/Kotlin**: BoltFFI vs UniFFI
- **WASM**: BoltFFI vs wasm-bindgen

All libraries wrap the exact same Rust code with identical public APIs, so the only variable is FFI overhead.

## Prerequisites

```bash
just setup-targets
```

For Android, set `ANDROID_NDK_HOME`.

## Why This Matters

FFI has inherent costs: crossing the language boundary, converting types, copying memory. UniFFI uses a runtime approach with serialization similar to JSON. BoltFFI generates specialized code at compile time that avoids most of this overhead.

These benchmarks isolate the FFI layer by using trivial Rust implementations (just constructing data or summing numbers).

## Test Data Structures

We test several struct types with increasing complexity:

**Location** (34 bytes, 6 fields)
```rust
struct Location {
    id: i64, lat: f64, lng: f64, rating: f64, review_count: i32, is_open: bool
}
```
Basic struct with only primitive fields.

**Trade** (65 bytes, 9 fields)
```rust
struct Trade {
    id: i64, symbol_id: i32, price: f64, quantity: i64,
    bid: f64, ask: f64, volume: i64, timestamp: i64, is_buy: bool
}
```
Larger struct representing financial data. Still only primitives.

**Particle** (81 bytes, 10 fields)
```rust
struct Particle {
    id: i64, x: f64, y: f64, z: f64, vx: f64, vy: f64, vz: f64,
    mass: f64, charge: f64, active: bool
}
```
Physics simulation data. Many f64 fields.

**SensorReading** (61 bytes, 9 fields)
```rust
struct SensorReading {
    sensor_id: i64, timestamp: i64, temperature: f64, humidity: f64,
    pressure: f64, light: f64, battery: f64, signal_strength: i32, is_valid: bool
}
```
IoT telemetry data.

**UserProfile** (variable size, 9 fields with heap data)
```rust
struct UserProfile {
    id: i64, name: String, email: String, bio: String, age: i32, score: f64,
    tags: Vec<String>, scores: Vec<i32>, is_active: bool
}
```
Contains three String fields, a `Vec<String>`, and a `Vec<i32>`. String handling and nested collections are where FFI's serialization overhead becomes most apparent.

## Benchmark Categories

### Call Overhead
- `noop`: Empty function. Pure FFI call cost with zero data transfer.

### Primitives
- `echo_i32`, `echo_f64`: Round-trip a single number.
- `add`, `multiply`: Arithmetic with two inputs and one output.
- `inc_u64`: Mutate a value through a mutable slice.

### Strings
- `echo_string_small`: 5-character string round-trip.
- `echo_string_1k`: 1,000-character string round-trip.

Strings require UTF-8 validation, length calculation, and memory allocation. The overhead gap narrows with size because memcpy eventually dominates.

### Struct Generation (Rust → Swift/Kotlin)
- `generate_locations_1k`, `generate_locations_10k`
- `generate_trades_1k`, `generate_trades_10k`
- `generate_particles_1k`, `generate_particles_10k`
- `generate_sensors_1k`, `generate_sensors_10k`
- `generate_user_profiles_100`, `generate_user_profiles_1k`

Rust creates vectors of structs and returns them. This measures serialization cost. UserProfile is particularly expensive because each item contains multiple heap-allocated strings.

### Struct Consumption (Swift/Kotlin → Rust)
- `sum_ratings`, `process_locations`: Pass Location vectors to Rust.
- `sum_trade_volumes`: Pass Trade vectors to Rust.
- `sum_particle_masses`: Pass Particle vectors to Rust.
- `avg_sensor_temp`: Pass SensorReading vectors to Rust.
- `sum_user_scores`, `count_active_users`: Pass UserProfile vectors to Rust.

This measures deserialization cost.

### Primitive Vectors
- `generate_i32_vec_10k`, `generate_i32_vec_100k`: Create Vec<i32>.
- `sum_i32_vec_10k`, `sum_i32_vec_100k`: Consume Vec<i32>.
- `generate_f64_vec_10k`, `sum_f64_vec_10k`: Same for f64.
- `generate_bytes_64k`: Raw byte array (64KB).

### Classes (Stateful Objects)
- `counter_increment`: Create a Counter object in Rust, call increment() 1,000 times from Swift/Kotlin, then call get().
- `datastore_add`: Create a DataStore, add 1,000 DataPoint structs.
- `accumulator`: Create an Accumulator, call add() 1,000 times, get(), reset().

These measure the cost of holding a Rust object handle and making repeated method calls across the FFI boundary.

### Enums
- `simple_enum`: C-style enum (Direction: North/South/East/West).
- `data_enum_input`: Enum with associated data (Status::InProgress(i32), Status::Completed(i32)).
- `find_even`: Returns Option<i32>. Tests nullable type handling.

### Async Functions
- `async_add`: Simple async function that adds two integers.

Measures async function call overhead including task spawning and completion handling.

### Callbacks (Foreign Traits)
- `callback_100`, `callback_1k`: Create a DataConsumer in Rust, set a DataProvider implemented in Swift/Kotlin, call computeSum() which iterates through all items.

Measures the cost of Rust calling back into Swift/Kotlin. Each iteration involves:
1. Call provider.getCount() from Rust → Swift/Kotlin
2. Loop calling provider.getItem(i) for each item (100 or 1000 calls)
3. Deserialize each DataPoint struct returned from Swift/Kotlin

## Running the Benchmarks

### Swift (macOS)

```bash
just bench-swift
```

### Kotlin (JVM via JMH)

```bash
just bench-kotlin
```

Report: `kotlin-jvm-bench/build/results/jmh/report.txt`

### iOS

```bash
just bench-build-ios
# Then open ios-app/ in Xcode
```

### Android

```bash
just bench-build-android
# Then open android-app/ in Android Studio
```

### WASM (Node.js)

```bash
just bench-wasm
```

## Results

These are actual results from running `just bench-swift` on Apple M3 chip:

| Benchmark | BoltFFI | UniFFI | Speedup |
|-----------|--------:|-------:|--------:|
| noop | <1 ns | 1,416 ns | >1000x |
| echo_i32 | <1 ns | 1,416 ns | >1000x |
| echo_string_small | 125 ns | 4,292 ns | 34x |
| echo_string_1k | 10,209 ns | 14,292 ns | 1.4x |
| generate_locations_1k | 4,167 ns | 1,276,333 ns | 306x |
| generate_locations_10k | 62,542 ns | 12,817,000 ns | 205x |
| generate_trades_1k | 12,208 ns | 1,920,000 ns | 157x |
| generate_user_profiles_100 | 65,125 ns | 505,250 ns | 7.8x |
| generate_user_profiles_1k | 701,604 ns | 5,174,792 ns | 7.4x |
| sum_i32_vec_10k | 833 ns | 69,959 ns | 84x |
| counter_increment (1k calls) | 1,083 ns | 1,388,895 ns | 1,282x |
| datastore_add (1k items) | 54,125 ns | 2,911,833 ns | 54x |
| process_locations_1k | 542 ns | 43,125 ns | 80x |
| callback_100 | 14,834 ns | 203,791 ns | 13.7x |
| callback_1k | 142,959 ns | 1,970,291 ns | 13.8x |

### WASM (Node.js)

Results from `just bench-wasm` on Apple M3:

| Benchmark | BoltFFI | wasm-bindgen | Speedup |
|-----------|--------:|-------------:|--------:|
| noop | 2 ns | 2 ns | 1x |
| echo_i32 | 2 ns | 2 ns | 1x |
| echo_f64 | 2 ns | 2 ns | 1x |
| add | 2 ns | 2 ns | 1x |
| multiply | 2 ns | 2 ns | 1x |
| echo_string_200 | 487 ns | 763 ns | 1.6x |
| echo_string_1k | 806 ns | 2,921 ns | 3.6x |
| generate_string_1k | 231 ns | 241 ns | 1x |
| generate_locations_100 | 2,199 ns | 283,753 ns | 129x |
| generate_locations_1k | 21,931 ns | 4,037,879 ns | 184x |
| generate_trades_100 | 5,595 ns | 616,253 ns | 110x |
| generate_trades_1k | 42,015 ns | 5,781,767 ns | 138x |
| generate_particles_100 | 3,117 ns | 748,287 ns | 240x |
| generate_particles_1k | 29,886 ns | 13,532,530 ns | 453x |
| generate_i32_vec_1k | 623 ns | 559 ns | -1.1x |
| generate_i32_vec_10k | 3,667 ns | 3,493 ns | 1x |
| generate_bytes_64k | 2,973 ns | 2,973 ns | 1x |
| roundtrip_locations_100 | 15,467 ns | 24,587 ns | 1.6x |
| roundtrip_i32_vec_1k | 1,305 ns | 1,228 ns | -1.1x |
| counter_increment_1k | 2,382 ns | 2,594 ns | 1.1x |
| datastore_add_1k | 91,226 ns | 115,574 ns | 1.3x |
| accumulator_1k | 14,096 ns | 13,778 ns | 1x |
| find_even_100 | 172 ns | 173 ns | 1x |
| async_add | 243 ns | 327 ns | 1.3x |

#### So who wins?

1. For pure primitives (integers, floats, scalars), both tie at ~2ns.

2. For strings, BoltFFI is 1.6-3.6x faster.

3. For structured data (records, arrays of structs), BoltFFI is **110-453x faster**.

4. For primitive vectors (`Vec<i32>`, `Vec<u8>`), both tie.

BoltFFI wins for real world mixed data, and ties or a bit slower with wasm-bindgen on scalar types.
