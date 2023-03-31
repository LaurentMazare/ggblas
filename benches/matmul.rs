#![feature(test)]

extern crate test;
use test::{black_box, Bencher};
use threadpool::ThreadPool;

use rblas::{ggml_matmul, matmul, Tensor};

const M: usize = 6;
const N: usize = 768 * 3;
const K: usize = 768;

#[bench]
fn bench_mkl(bench: &mut Bencher) {
    let a = Tensor {
        shape: vec![M, K],
        data: vec![0.0; M * K],
    };
    let b = Tensor {
        shape: vec![N, K],
        data: vec![0.0; N * K],
    };
    let mut c = Tensor {
        shape: vec![M, N],
        data: vec![0.0; M * N],
    };
    bench.iter(|| black_box(matmul::<true>(&a, &b, &mut c)));
}

#[bench]
fn bench_ggml(bench: &mut Bencher) {
    let pool = ThreadPool::new(num_cpus::get());
    let a = Tensor {
        shape: vec![M, K],
        data: vec![0.0; M * K],
    };
    let b = Tensor {
        shape: vec![N, K],
        data: vec![0.0; N * K],
    };
    let mut c = Tensor {
        shape: vec![M, N],
        data: vec![0.0; M * N],
    };
    bench.iter(|| black_box(ggml_matmul::<true>(&a, &b, &mut c, &pool)));
}
