use rand::distributions::{Alphanumeric, Distribution, Standard};
use rand::{thread_rng, Rng};

pub fn gen_random_int_vec<T>(size: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    let mut arr = Vec::new();
    for _ in 0..size {
        arr.push(gen_random_int::<T>())
    }

    arr
}

pub fn gen_random_int<T>() -> T
where
    Standard: Distribution<T>,
{
    thread_rng().gen()
}

pub fn gen_random_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

pub mod reg_db {
    use uuid::Uuid;

    hmdb::schema! {
        RegularDb {
            table1: <u8, String>,
            table2: <i32, u8>,
            table3: <u128, String>,
            table4: <isize, u128>,
            table5: <Vec<u8>, Uuid>,
            table6: <Uuid, usize>
        }
    }
}

pub mod large_db {
    hmdb::schema! {
        LargeDb {
            table1: <u8, String>,
            table2: <u8, String>,
            table3: <u8, String>,
            table4: <u8, String>,
            table5: <u8, String>,
            table6: <u8, String>,
            table7: <u8, String>,
            table8: <u8, String>,
            table9: <u8, String>,
            table10: <u8, String>,
            table11: <u8, String>,
            table12: <u8, String>,
            table13: <u8, String>,
            table14: <u8, String>,
            table15: <u8, String>,
            table16: <u8, String>,
            table17: <u8, String>,
            table18: <u8, String>,
            table19: <u8, String>,
            table20: <u8, String>,
            table21: <u8, String>,
            table22: <u8, String>,
            table23: <u8, String>,
            table24: <u8, String>,
            table25: <u8, String>,
            table26: <u8, String>,
            table27: <u8, String>,
            table28: <u8, String>,
            table29: <u8, String>,
            table30: <u8, String>,
            table31: <u8, String>,
            table32: <u8, String>,
            table33: <u8, String>,
            table34: <u8, String>,
            table35: <u8, String>,
            table36: <u8, String>,
            table37: <u8, String>,
            table38: <u8, String>,
            table39: <u8, String>,
            table40: <u8, String>
        }
    }
}

pub mod small_db {
    use uuid::Uuid;

    hmdb::schema! {
        SmallDb {
            table1: <Uuid, u32>
        }
    }
}
