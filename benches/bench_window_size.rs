use ark_bls12_381::G1Affine;
use ark_ff::BigInteger;
use ark_msm::{msm::VariableBaseMSM, utils::generate_msm_inputs};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mcl_rust as mcl;

fn to_ptr_fp(x: &ark_bls12_381::Fq) -> *const mcl::Fp {
    let p: *const _ = x;
    let yp = p as *const mcl::Fp;
    yp
}

fn to_fp(x: &ark_bls12_381::Fq) -> mcl::Fp {
    unsafe { (*to_ptr_fp(&x)).clone() }
}

fn to_g1(p: &ark_bls12_381::G1Affine) -> mcl::G1 {
    unsafe {
        let mut ret = mcl::G1::uninit();
        ret.x = to_fp(&p.x);
        ret.y = to_fp(&p.y);
        ret.z.set_int(1);
        ret
    }
}

fn mod_to_fr(x: &ark_ff::biginteger::BigInteger256) -> mcl::Fr {
    unsafe {
        let mut ret = mcl::Fr::uninit();
        ret.set_little_endian_mod(&x.to_bytes_le());
        ret
    }
}

fn to_g1s(iv: &Vec<ark_bls12_381::G1Affine>) -> Vec<mcl::G1> {
    let mut ov: Vec<_> = Vec::new();
    let n = iv.len();
    ov.reserve(n);
    for i in 0..n {
        ov.push(to_g1(&iv[i]));
    }
    return ov;
}

fn mod_to_frs(iv: &Vec<ark_ff::biginteger::BigInteger256>) -> Vec<mcl::Fr> {
    let mut ov: Vec<_> = Vec::new();
    let n = iv.len();
    ov.reserve(n);
    for i in 0..n {
        ov.push(mod_to_fr(&iv[i]));
    }
    return ov;
}

fn bench_window_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("msm");
    group.sample_size(20);
    mcl::init(mcl::CurveType::BLS12_381);
    for size in 10..20 {
        let (point_vec, scalar_vec) = generate_msm_inputs::<G1Affine>(1 << size);
        let point_vec = black_box(point_vec);
        let scalar_vec = black_box(scalar_vec);
        let mcl_xs = to_g1s(&point_vec);
        let mcl_ys = mod_to_frs(&scalar_vec);

        for window_size in (size - 3)..(size + 3) {
            let input = (size, window_size);
            let benchmark_id =
                BenchmarkId::new("ArkMSM", format!("k={}, ws={}", size, window_size));
            group.bench_with_input(benchmark_id, &input, |b, _input| {
                b.iter(|| {
                    let _ = VariableBaseMSM::multi_scalar_mul_custom(
                        &point_vec,
                        &scalar_vec,
                        window_size,
                        2048,
                        256,
                        true,
                    );
                })
            });
        }

        {
            let input = (size, 0);
            let benchmark_id = BenchmarkId::new("mcl", format!("k={}", size));
            let mut g1 = mcl::G1::zero();
            group.bench_with_input(benchmark_id, &input, |b, _input| {
                b.iter(|| {
                    mcl::G1::mul_vec(&mut g1, &mcl_xs, &mcl_ys);
                })
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_window_size);
criterion_main!(benches);
