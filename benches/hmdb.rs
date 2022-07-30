use crate::schemas::large_db::LargeDb;
use crate::schemas::reg_db::RegularDb;
use crate::schemas::small_db::SmallDb;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hmdb::log::Reader;
use hmdb::test_utils::{test_db, test_dbs_folder};
use rand::distributions::{Alphanumeric, Distribution, Standard};
use rand::{thread_rng, Rng};
use std::fs;
use uuid::Uuid;

mod schemas;

const INSERT_OP: u64 = 100;
const INSERT_OPS: [u64; 5] = [
    INSERT_OP,
    INSERT_OP * 10,
    INSERT_OP * 20,
    INSERT_OP * 50,
    INSERT_OP * 100,
];

fn gen_random_int_vec<T>(size: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    let mut arr = Vec::new();
    for _ in 0..size {
        arr.push(gen_random_int::<T>())
    }

    arr
}

fn gen_random_int<T>() -> T
where
    Standard: Distribution<T>,
{
    thread_rng().gen()
}

fn gen_random_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

fn small_db_ops(c: &mut Criterion) {
    let mut dbs = vec![];

    c.bench_function("init empty small db", |b| {
        let path = test_db();
        SmallDb::init(&path).unwrap();

        b.iter(|| SmallDb::init(black_box(&path)).unwrap());
    });

    let mut insert_db_group = c.benchmark_group("insert small db");
    for size in INSERT_OPS {
        insert_db_group.throughput(Throughput::Elements(size));
        insert_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let path = test_db();
            dbs.push(path.clone());
            let db = SmallDb::init(&path).unwrap();

            let random_uuid = Uuid::new_v4();
            let random_u32: u32 = gen_random_int();

            b.iter(|| {
                db.table1
                    .insert(black_box(random_uuid.clone()), black_box(random_u32))
                    .unwrap();
            });

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    insert_db_group.finish();

    let mut init_db_group = c.benchmark_group("init small db");
    for (db_path, size) in dbs.iter().zip(INSERT_OPS) {
        init_db_group.throughput(Throughput::Elements(size));
        init_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            b.iter(|| {
                SmallDb::init(black_box(db_path)).unwrap();
            });

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    init_db_group.finish();

    let mut access_db_group = c.benchmark_group("get small db");
    for (db_path, size) in dbs.iter().zip(INSERT_OPS) {
        access_db_group.throughput(Throughput::Elements(size));
        access_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let db = SmallDb::init(db_path).unwrap();

            b.iter(|| db.table1.get(black_box(&Uuid::new_v4())).unwrap());

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    access_db_group.finish();

    c.bench_function("delete small db", |b| {
        let db = SmallDb::init(db_path).unwrap();

        b.iter(|| db.table1.delete(black_box(Uuid::new_v4())).unwrap());

        fs::remove_dir_all(&test_dbs_folder()).unwrap();
    });
}

fn regular_db_ops(c: &mut Criterion) {
    let mut dbs = vec![];

    c.bench_function("init empty regular db", |b| {
        let path = test_db();
        RegularDb::init(&path).unwrap();

        b.iter(|| RegularDb::init(black_box(&path)).unwrap());

        fs::remove_dir_all(&test_dbs_folder()).unwrap();
    });

    let mut insert_db_group = c.benchmark_group("insert regular db");
    for size in INSERT_OPS {
        insert_db_group.throughput(Throughput::Elements(size));
        insert_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let path = test_db();
            dbs.push(path.clone());
            let db = RegularDb::init(&path).unwrap();

            let random_u8: u8 = gen_random_int();
            let random_i32: i32 = gen_random_int();
            let random_u128: u128 = gen_random_int();
            let random_isize: isize = gen_random_int();
            let random_vec_u8: Vec<u8> = gen_random_int_vec(10);
            let random_uuid = Uuid::new_v4();
            let random_usize: usize = gen_random_int();
            let random_string = gen_random_string();

            b.iter(|| {
                db.table1
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table2
                    .insert(black_box(random_i32), black_box(random_u8))
                    .unwrap();
                db.table3
                    .insert(black_box(random_u128), black_box(random_string.clone()))
                    .unwrap();
                db.table4
                    .insert(black_box(random_isize), black_box(random_u128))
                    .unwrap();
                db.table5
                    .insert(black_box(random_vec_u8.clone()), black_box(random_uuid))
                    .unwrap();
                db.table6
                    .insert(black_box(random_uuid), black_box(random_usize))
                    .unwrap();
            });

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    insert_db_group.finish();

    let mut init_db_group = c.benchmark_group("init regular db");
    for (db_path, size) in dbs.iter().zip(INSERT_OPS) {
        init_db_group.throughput(Throughput::Elements(size));
        init_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            b.iter(|| {
                RegularDb::init(black_box(db_path)).unwrap();
            });

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    init_db_group.finish();

    let mut access_db_group = c.benchmark_group("get regular db");
    for (db_path, size) in dbs.iter().zip(INSERT_OPS) {
        access_db_group.throughput(Throughput::Elements(size));
        access_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let db = RegularDb::init(db_path).unwrap();

            b.iter(|| db.table1.get(&gen_random_int()).unwrap());

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    access_db_group.finish();

    c.bench_function("delete regular db", |b| {
        let db = RegularDb::init(db_path).unwrap();

        b.iter(|| db.table1.delete(black_box(gen_random_int())).unwrap());

        fs::remove_dir_all(&test_dbs_folder()).unwrap();
    });
}

fn large_db_ops(c: &mut Criterion) {
    let mut dbs = vec![];

    c.bench_function("init empty large db", |b| {
        let path = test_db();
        LargeDb::init(&path).unwrap();

        b.iter(|| LargeDb::init(black_box(&path)).unwrap());

        fs::remove_dir_all(&test_dbs_folder()).unwrap();
    });

    let mut insert_db_group = c.benchmark_group("insert large db");
    for size in INSERT_OPS {
        insert_db_group.throughput(Throughput::Elements(size));
        insert_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let path = test_db();
            dbs.push(path.clone());
            let db = LargeDb::init(&path).unwrap();

            let random_u8: u8 = gen_random_int();
            let random_string = gen_random_string();

            b.iter(|| {
                db.table1
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table2
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table3
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table4
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table5
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table6
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table7
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table8
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table9
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table11
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table12
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table13
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table14
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table15
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table16
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table17
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table18
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table19
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table21
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table22
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table23
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table24
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table25
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table26
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table27
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table28
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table29
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table30
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table31
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table32
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table33
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table34
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table35
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table36
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table37
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table38
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table39
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
                db.table40
                    .insert(black_box(random_u8), black_box(random_string.clone()))
                    .unwrap();
            });

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    insert_db_group.finish();

    let mut init_db_group = c.benchmark_group("init regular db");
    for (db_path, size) in dbs.iter().zip(INSERT_OPS) {
        init_db_group.throughput(Throughput::Elements(size));
        init_db_group.bench_function(BenchmarkId::from_parameter(size),|b| {
            b.iter(|| {
                LargeDb::init(black_box(db_path)).unwrap();
            });

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    init_db_group.finish();

    let mut access_db_group = c.benchmark_group("get large db");
    for (db_path, size) in dbs.iter().zip(INSERT_OPS) {
        access_db_group.throughput(Throughput::Elements(size));
        access_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let db = LargeDb::init(db_path).unwrap();

            b.iter(|| db.table1.get(black_box(&gen_random_int())).unwrap());

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    access_db_group.finish();

    c.bench_function("delete large db", |b| {
        let db = LargeDb::init(db_path).unwrap();

        b.iter(|| db.table1.delete(black_box(gen_random_int())).unwrap());

        fs::remove_dir_all(&test_dbs_folder()).unwrap();
    });
}

fn sled_ops(c: &mut Criterion) {
    let mut dbs = vec![];

    c.bench_function("init sled empty db", |b| {
        let path = test_db();
        sled::open(&path).unwrap();

        b.iter(|| sled::open(black_box(&path)).unwrap());

        fs::remove_dir_all(&test_dbs_folder()).unwrap();
    });

    let mut insert_db_group = c.benchmark_group("insert sled db");
    for size in INSERT_OPS {
        insert_db_group.throughput(Throughput::Elements(size));
        insert_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let path = test_db();
            dbs.push(path.clone());
            let db = sled::open(&path).unwrap();

            b.iter(|| {
                db.insert(black_box(gen_random_int_vec::<u8>(100)), black_box(vec![]))
                    .unwrap();
            });

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    insert_db_group.finish();

    let mut access_db_group = c.benchmark_group("get sled db");
    for (db_path, size) in dbs.iter().zip(INSERT_OPS) {
        access_db_group.throughput(Throughput::Elements(size));
        access_db_group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let db = sled::open(&db_path).unwrap();

            b.iter(|| {
                db.get(black_box(gen_random_int_vec::<u8>(100))).unwrap();
            });

            fs::remove_dir_all(&test_dbs_folder()).unwrap();
        });
    }
    access_db_group.finish();

    c.bench_function("remove sled db", |b| {
        let path = test_db();
        let db = sled::open(&path).unwrap();

        b.iter(|| {
            db.remove(black_box(gen_random_int_vec::<u8>(100))).unwrap();
        });

        fs::remove_dir_all(&test_dbs_folder()).unwrap();
    });
}

criterion_group!(
    benches,
    large_db_ops,
    regular_db_ops,
    small_db_ops,
    sled_ops
);
criterion_main!(benches);
