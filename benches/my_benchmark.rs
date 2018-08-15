#[macro_use]
extern crate criterion;
extern crate iconicvc;
extern crate rand;

use criterion::Criterion;
use iconicvc::Container;
use rand::prelude::*;

fn create_container(till_price: u64, till_size: u64) -> Container {
    let mut container = Container::new();

    for i in 1..till_price {
        for j in 1..till_size {
            container.add(i, j, "".to_owned());
        }
    }

    container
}

macro_rules! quote_bench_add {
    ($criterion: expr, $max_price:expr, $max_size: expr, $name: expr) => {
        let container = create_container($max_price, $max_size);

        let mut rng = thread_rng();
        let i = rng.gen_range(0, $max_price);
        let j = rng.gen_range(0, $max_size);

        $criterion.bench_function($name, move |b| {
            b.iter_with_setup(
                || (container.clone().without(i, j), i, j),
                |mut data| data.0.add(data.1, data.2, "".to_owned()),
            )
        });
    }
}

fn add_light_benchmark(c: &mut Criterion) {
    quote_bench_add!(c, 100, 20, "add 2000");
}

fn add_medium_benchmark(c: &mut Criterion) {
    quote_bench_add!(c, 100, 70, "add 7000");
}

fn add_hard_benchmark(c: &mut Criterion) {
    quote_bench_add!(c, 200, 100, "add 20000");
}

fn process_benchmark(c: &mut Criterion) {
    let c1 = create_container(100, 70);

    let calculate_sum = |n: u64| -> u64 { n * (n + 1) / 2 };
    // This is very rough and actually asks more elements: 1, 20, 167, 3587
    let consume = |to_consume: u64| -> (u64, u64) { (100, calculate_sum(to_consume)) };

    let inputs = vec![consume(1), consume(20), consume(100), consume(500)];

    c.bench_function_over_inputs(
        "process",
        move |b, &input| {
            b.iter_with_setup(|| c1.clone(), |mut c| c.process(input.0, input.1));
        },
        inputs,
    );
}

criterion_group!(
    benches,
    add_light_benchmark,
    add_medium_benchmark,
    add_hard_benchmark,
    process_benchmark,
);
criterion_main!(benches);
