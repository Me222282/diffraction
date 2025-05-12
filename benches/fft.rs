use std::time::Instant;

use rand::random;

use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
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

fn rec_bench(b: &mut Bencher, data: &(&[Complex<f32>], &[Complex<f32>]))
{
    b.iter_custom(|iters|
    {
        let mut copy = vec![Complex::<f32>::ZERO; data.1.len()];
        copy.copy_from_slice(data.1);
        let start = Instant::now();
        for _ in 0..iters
        {
            fft_recursive(data.0, copy.as_mut_slice());
        }
        return start.elapsed();
    } )
}
fn it_bench(b: &mut Bencher, data: &(&[Complex<f32>], &[Complex<f32>]))
{
    b.iter_custom(|iters|
    {
        let mut copy = vec![Complex::<f32>::ZERO; data.1.len()];
        copy.copy_from_slice(data.1);
        let start = Instant::now();
        for _ in 0..iters
        {
            fft_iterative(data.0, copy.as_mut_slice());
        }
        return start.elapsed();
    } )
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("FFT");
    
    // size 8
    let size = 8;
    let data = gen_data(size);
    let wn = compute_nth_roots::<f32>(size);
    
    group.bench_with_input(
        BenchmarkId::new("Recursive", size),
        &(wn.as_ref(), data.as_ref()),
        rec_bench);
    group.bench_with_input(
        BenchmarkId::new("Iterative", size),
        &(wn.as_ref(), data.as_ref()),
        it_bench);
    
    // size 16
    let size = 16;
    let data = gen_data(size);
    let wn = compute_nth_roots::<f32>(size);
    
    group.bench_with_input(
        BenchmarkId::new("Recursive", size),
        &(wn.as_ref(), data.as_ref()),
        rec_bench);
    group.bench_with_input(
        BenchmarkId::new("Iterative", size),
        &(wn.as_ref(), data.as_ref()),
        it_bench);
    
    // size 64
    let size = 64;
    let data = gen_data(size);
    let wn = compute_nth_roots::<f32>(size);
    
    group.bench_with_input(
        BenchmarkId::new("Recursive", size),
        &(wn.as_ref(), data.as_ref()),
        rec_bench);
    group.bench_with_input(
        BenchmarkId::new("Iterative", size),
        &(wn.as_ref(), data.as_ref()),
        it_bench);
    
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);