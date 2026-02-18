package com.boltffi.bench

import com.example.bench_boltffi.*

fun runAllBenchmarks(onProgress: (name: String, progress: Float) -> Unit): List<BenchmarkResult> {
    val boltffiLocations1k = generateLocations(1000)
    val boltffiLocations10k = generateLocations(10000)
    val boltffiTrades1k = generateTrades(1000)
    val boltffiParticles1k = generateParticles(1000)
    val boltffiSensors1k = generateSensorReadings(1000)
    val boltffiI32Vec10k = generateI32Vec(10000)
    val boltffiF64Vec10k = generateF64Vec(10000)
    val boltffiUsers100 = generateUserProfiles(100)
    val boltffiUsers1k = generateUserProfiles(1000)

    val benchmarks = listOf(
        bench("noop", "1. FFI Overhead", 10000,
            boltffi = { noop() },
            uniffi = { uniffi.bench_uniffi.noop() }),
        bench("echo_i32", "1. FFI Overhead", 10000,
            boltffi = { echoI32(42) },
            uniffi = { uniffi.bench_uniffi.echoI32(42) }),
        bench("add", "1. FFI Overhead", 10000,
            boltffi = { add(100, 200) },
            uniffi = { uniffi.bench_uniffi.add(100, 200) }),

        bench("echo_string_small", "2. Strings", 5000,
            boltffi = { echoString("hello") },
            uniffi = { uniffi.bench_uniffi.echoString("hello") }),
        bench("echo_string_1k", "2. Strings", 2000,
            boltffi = { echoString("x".repeat(1000)) },
            uniffi = { uniffi.bench_uniffi.echoString("x".repeat(1000)) }),

        bench("generate_locations_1k", "3. Rust→Kotlin Blittable", 500,
            boltffi = { generateLocations(1000) },
            uniffi = { uniffi.bench_uniffi.generateLocations(1000) }),
        bench("generate_locations_10k", "3. Rust→Kotlin Blittable", 50,
            boltffi = { generateLocations(10000) },
            uniffi = { uniffi.bench_uniffi.generateLocations(10000) }),
        bench("generate_trades_1k", "3. Rust→Kotlin Blittable", 500,
            boltffi = { generateTrades(1000) },
            uniffi = { uniffi.bench_uniffi.generateTrades(1000) }),
        bench("generate_particles_1k", "3. Rust→Kotlin Blittable", 500,
            boltffi = { generateParticles(1000) },
            uniffi = { uniffi.bench_uniffi.generateParticles(1000) }),
        bench("generate_sensors_1k", "3. Rust→Kotlin Blittable", 500,
            boltffi = { generateSensorReadings(1000) },
            uniffi = { uniffi.bench_uniffi.generateSensorReadings(1000) }),
        bench("generate_i32_vec_10k", "3. Rust→Kotlin Blittable", 500,
            boltffi = { generateI32Vec(10000) },
            uniffi = { uniffi.bench_uniffi.generateI32Vec(10000) }),
        bench("generate_f64_vec_10k", "3. Rust→Kotlin Blittable", 500,
            boltffi = { generateF64Vec(10000) },
            uniffi = { uniffi.bench_uniffi.generateF64Vec(10000) }),
        bench("generate_bytes_64k", "3. Rust→Kotlin Blittable", 200,
            boltffi = { generateBytes(65536) },
            uniffi = { uniffi.bench_uniffi.generateBytes(65536) }),

        bench("process_locations_1k", "4. Kotlin→Rust Blittable", 1000,
            boltffi = { processLocations(boltffiLocations1k) },
            uniffi = { uniffi.bench_uniffi.processLocations(uniffi.bench_uniffi.generateLocations(1000)) }),
        bench("process_locations_10k", "4. Kotlin→Rust Blittable", 100,
            boltffi = { processLocations(boltffiLocations10k) },
            uniffi = { uniffi.bench_uniffi.processLocations(uniffi.bench_uniffi.generateLocations(10000)) }),
        bench("sum_ratings_1k", "4. Kotlin→Rust Blittable", 1000,
            boltffi = { sumRatings(boltffiLocations1k) },
            uniffi = { uniffi.bench_uniffi.sumRatings(uniffi.bench_uniffi.generateLocations(1000)) }),
        bench("sum_ratings_10k", "4. Kotlin→Rust Blittable", 100,
            boltffi = { sumRatings(boltffiLocations10k) },
            uniffi = { uniffi.bench_uniffi.sumRatings(uniffi.bench_uniffi.generateLocations(10000)) }),
        bench("sum_trade_volumes_1k", "4. Kotlin→Rust Blittable", 1000,
            boltffi = { sumTradeVolumes(boltffiTrades1k) },
            uniffi = { uniffi.bench_uniffi.sumTradeVolumes(uniffi.bench_uniffi.generateTrades(1000)) }),
        bench("sum_particle_masses_1k", "4. Kotlin→Rust Blittable", 1000,
            boltffi = { sumParticleMasses(boltffiParticles1k) },
            uniffi = { uniffi.bench_uniffi.sumParticleMasses(uniffi.bench_uniffi.generateParticles(1000)) }),
        bench("avg_sensor_temp_1k", "4. Kotlin→Rust Blittable", 1000,
            boltffi = { avgSensorTemperature(boltffiSensors1k) },
            uniffi = { uniffi.bench_uniffi.avgSensorTemperature(uniffi.bench_uniffi.generateSensorReadings(1000)) }),
        bench("sum_i32_vec_10k", "4. Kotlin→Rust Blittable", 500,
            boltffi = { sumI32Vec(boltffiI32Vec10k) },
            uniffi = { uniffi.bench_uniffi.sumI32Vec(uniffi.bench_uniffi.generateI32Vec(10000)) }),
        bench("sum_f64_vec_10k", "4. Kotlin→Rust Blittable", 500,
            boltffi = { sumF64Vec(boltffiF64Vec10k) },
            uniffi = { uniffi.bench_uniffi.sumF64Vec(uniffi.bench_uniffi.generateF64Vec(10000)) }),

        bench("generate_user_profiles_100", "5. Rust→Kotlin Complex", 200,
            boltffi = { generateUserProfiles(100) },
            uniffi = { uniffi.bench_uniffi.generateUserProfiles(100) }),
        bench("generate_user_profiles_1k", "5. Rust→Kotlin Complex", 20,
            boltffi = { generateUserProfiles(1000) },
            uniffi = { uniffi.bench_uniffi.generateUserProfiles(1000) }),

        bench("sum_user_scores_100", "6. Kotlin→Rust Complex", 500,
            boltffi = { sumUserScores(boltffiUsers100) },
            uniffi = { uniffi.bench_uniffi.sumUserScores(uniffi.bench_uniffi.generateUserProfiles(100)) }),
        bench("sum_user_scores_1k", "6. Kotlin→Rust Complex", 50,
            boltffi = { sumUserScores(boltffiUsers1k) },
            uniffi = { uniffi.bench_uniffi.sumUserScores(uniffi.bench_uniffi.generateUserProfiles(1000)) }),
        bench("count_active_users_100", "6. Kotlin→Rust Complex", 500,
            boltffi = { countActiveUsers(boltffiUsers100) },
            uniffi = { uniffi.bench_uniffi.countActiveUsers(uniffi.bench_uniffi.generateUserProfiles(100)) }),
        bench("count_active_users_1k", "6. Kotlin→Rust Complex", 50,
            boltffi = { countActiveUsers(boltffiUsers1k) },
            uniffi = { uniffi.bench_uniffi.countActiveUsers(uniffi.bench_uniffi.generateUserProfiles(1000)) }),

        bench("counter_increment_1k (mutex)", "7. Classes", 100,
            boltffi = {
                Counter().use { c -> repeat(1000) { c.increment() } }
            },
            uniffi = {
                uniffi.bench_uniffi.Counter().use { c -> repeat(1000) { c.increment() } }
            }),
        benchBoltffiOnly("counter_increment_1k (single_threaded)", "7. Classes (BoltFFI-only)", 100) {
            CounterSingleThreaded().use { c -> repeat(1000) { c.increment() } }
        },
        bench("datastore_add_1k", "7. Classes", 50,
            boltffi = {
                DataStore().use { s ->
                    repeat(1000) { i ->
                        s.add(DataPoint(i.toDouble(), i.toDouble() * 2.0, i.toLong()))
                    }
                }
            },
            uniffi = {
                uniffi.bench_uniffi.DataStore().use { s ->
                    repeat(1000) { i ->
                        s.add(uniffi.bench_uniffi.DataPoint(i.toDouble(), i.toDouble() * 2.0, i.toLong()))
                    }
                }
            }),
        bench("accumulator_1k (mutex)", "7. Classes", 100,
            boltffi = {
                Accumulator().use { a ->
                    repeat(1000) { i -> a.add(i.toLong()) }
                    a.get()
                    a.reset()
                }
            },
            uniffi = {
                uniffi.bench_uniffi.Accumulator().use { a ->
                    repeat(1000) { i -> a.add(i.toLong()) }
                    a.get()
                    a.reset()
                }
            }),
        benchBoltffiOnly("accumulator_1k (single_threaded)", "7. Classes (BoltFFI-only)", 100) {
            AccumulatorSingleThreaded().use { a ->
                repeat(1000) { i -> a.add(i.toLong()) }
                a.get()
                a.reset()
            }
        },

        bench("simple_enum", "8. Enums", 5000,
            boltffi = {
                oppositeDirection(Direction.NORTH)
                directionToDegrees(Direction.EAST)
            },
            uniffi = {
                uniffi.bench_uniffi.oppositeDirection(uniffi.bench_uniffi.Direction.NORTH)
                uniffi.bench_uniffi.directionToDegrees(uniffi.bench_uniffi.Direction.EAST)
            }),
        bench("data_enum_input", "8. Enums", 5000,
            boltffi = {
                getStatusProgress(TaskStatus.InProgress(50))
                isStatusComplete(TaskStatus.Completed(100))
            },
            uniffi = {
                getStatusProgress(TaskStatus.InProgress(50))
                isStatusComplete(TaskStatus.Completed(100))
            }),

        bench("find_even_100", "9. Options", 1000,
            boltffi = { repeat(100) { i -> findEven(i) } },
            uniffi = { repeat(100) { i -> uniffi.bench_uniffi.findEven(i) } }),
    )

    return benchmarks.mapIndexed { index, b ->
        onProgress(b.name, index.toFloat() / benchmarks.size)
        b.run()
    }
}

fun runPhaseIsolation(): String {
    return "Phase isolation temporarily disabled (jbyteArray migration)"
}

private class BenchSpec(
    val name: String,
    val category: String,
    val iterations: Int,
    val boltffi: () -> Unit,
    val uniffi: () -> Unit,
) {
    fun run(): BenchmarkResult {
        repeat(10) { boltffi(); uniffi() }
        val boltffiNs = measureAvgNs(iterations, boltffi)
        val uniffiNs = measureAvgNs(iterations, uniffi)
        return BenchmarkResult(name, category, boltffiNs, uniffiNs)
    }
}

private fun bench(
    name: String,
    category: String,
    iterations: Int,
    boltffi: () -> Unit,
    uniffi: () -> Unit,
) = BenchSpec(name, category, iterations, boltffi, uniffi)

private fun benchBoltffiOnly(
    name: String,
    category: String,
    iterations: Int,
    boltffi: () -> Unit,
) = BenchSpec(name, category, iterations, boltffi, boltffi)

private inline fun measureAvgNs(iterations: Int, block: () -> Unit): Long {
    val start = System.nanoTime()
    repeat(iterations) { block() }
    val elapsed = System.nanoTime() - start
    return elapsed / iterations.toLong()
}
