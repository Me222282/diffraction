use std::time::Instant;

use rand::random;

use criterion::{criterion_group, criterion_main, measurement::WallTime, Bencher, BenchmarkGroup, BenchmarkId, Criterion};
use num::Complex;

use backend::*;

fn gen_data(size: usize) -> Box<[Complex<f32>]>
{
    let mut v = vec![Complex::<f32>::ZERO; size];
    for x in &mut v
    {
        *x = Complex::new(random::<f32>(), random::<f32>());
    }
    return v.into_boxed_slice();
}

fn rec_bench(b: &mut Bencher, data: &(&WCache<f32>, &[Complex<f32>], usize))
{
    b.iter_custom(|iters|
    {
        let mut copy = vec![Complex::<f32>::ZERO; data.1.len()];
        copy.copy_from_slice(data.1);
        let start = Instant::now();
        for _ in 0..iters
        {
            // fft_recursive(data.0, copy.as_mut_slice(), data.2);
        }
        return start.elapsed();
    } )
}
fn it_bench(b: &mut Bencher, data: &(&WCache<f32>, &[Complex<f32>], usize))
{
    b.iter_custom(|iters|
    {
        let mut copy = vec![Complex::<f32>::ZERO; data.1.len()];
        copy.copy_from_slice(data.1);
        let start = Instant::now();
        for _ in 0..iters
        {
            fft_iterative_v2(data.0, copy.as_mut_slice(), data.2);
        }
        return start.elapsed();
    } )
}
fn rec2_bench(b: &mut Bencher, data: &(&WCache<f32>, &[Complex<f32>], usize))
{
    b.iter_custom(|iters|
    {
        let mut copy = vec![Complex::<f32>::ZERO; data.1.len()];
        copy.copy_from_slice(data.1);
        let start = Instant::now();
        for _ in 0..iters
        {
            // fft_recursive_v2(data.0, copy.as_mut_slice(), data.2);
        }
        return start.elapsed();
    } )
}
fn it2_bench(b: &mut Bencher, data: &(&WCache<f32>, &[Complex<f32>], usize))
{
    b.iter_custom(|iters|
    {
        let mut copy = vec![Complex::<f32>::ZERO; data.1.len()];
        copy.copy_from_slice(data.1);
        let start = Instant::now();
        for _ in 0..iters
        {
            // fft_iterative_v3(data.0, copy.as_mut_slice());
        }
        return start.elapsed();
    } )
}

fn bench_all(group: &mut BenchmarkGroup<'_, WallTime>, power: usize)
{
    let size = 1 << power;
    let data = gen_data(size);
    let mut wn = WCache::<f32>::new(false);
    wn.ensure_max_power(power);
    
    group.bench_with_input(
        BenchmarkId::new("Recursive", size),
        &(&wn, data.as_ref(), power),
        rec_bench);
    // group.bench_with_input(
    //     BenchmarkId::new("Iterative", size),
    //     &(&wn, data.as_ref(), power),
    //     it_bench);
    group.bench_with_input(
        BenchmarkId::new("Recursive_V2", size),
        &(&wn, data.as_ref(), power),
        rec2_bench);
    group.bench_with_input(
        BenchmarkId::new("Iterative_V2", size),
        &(&wn, data.as_ref(), power),
        it_bench);
    group.bench_with_input(
        BenchmarkId::new("Iterative_V3", size),
        &(&wn, data.as_ref(), power),
        it2_bench);
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("FFT");
    
    // size 8
    bench_all(&mut group, 3);
    bench_all(&mut group, 4);
    bench_all(&mut group, 5);
    bench_all(&mut group, 7);
    bench_all(&mut group, 10);
    bench_all(&mut group, 16);
    
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);