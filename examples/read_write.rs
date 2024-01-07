extern crate db_shm_manager;
use db_shm_manager::{DoubleBufferedSharedMemory, errors::DbShmError};
use ndarray::*;
use ndarray_rand::RandomExt;
use rand::distributions::Uniform;

fn main() {
    match DoubleBufferedSharedMemory::<u8>::new("my_shared_mem", (1080, 1920, 3)) {
        Ok(mut dbshm) => {
            let shape = (1080, 1920, 3);
            let mut rng = rand::thread_rng();
            let write_array3: Array3<u8> = Array3::random_using(shape, Uniform::new(0u8, 255u8), &mut rng);
            let write_array: ArrayD<u8> = write_array3.into_dyn();
            match dbshm.write(&write_array) {
              Ok(_) => println!("Scrittura riuscita"),
              Err(DbShmError::InvalidSize(expected_size, actual_size)) => {
                println!("Invalid size provided. Expected: {}, Actual: {}", expected_size, actual_size);
              },
              Err(e) => eprintln!("Errore nella scrittura: {:?}", e),
            }

            match dbshm.read() {
              Ok(read_array) => println!("Read array: {:?}", read_array),
              Err(e) => eprintln!("Errore nella lettura: {:?}", e),
            }
        },
        Err(e) => eprintln!("Errore nell'inizializzazione della memoria condivisa: {:?}", e),
    }
}
