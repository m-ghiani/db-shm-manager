extern crate db_shm_manager;
use db_shm_manager::DoubleBufferedSharedMemory;
use ndarray::*;
use ndarray_rand::RandomExt;
use rand::distributions::Uniform;
use std::mem;

fn main() {
    // Creazione di DoubleBufferedSharedMemory
    let mut dbshm = match DoubleBufferedSharedMemory::<u8>::new("my_shared_mem", (1080, 1920, 3)) {
        Ok(dbshm) => dbshm,
        Err(e) => {
            eprintln!("Errore nell'inizializzazione della memoria condivisa: {:?}", e);
            return;
        }
    };

    let shape = (1080, 1920, 3);
    let mut rng = rand::thread_rng();
    let write_array3: Array3<u8> = Array3::random_using(shape, Uniform::new(0u8, 255u8), &mut rng);
    let write_array: ArrayD<u8> = write_array3.into_dyn();

    // Scrittura nell'array
    if let Err(e) = dbshm.write(&write_array) {
        eprintln!("Errore nella scrittura: {:?}", e);
    }

    // Lettura dall'array
    match dbshm.read() {
        Ok(read_array) => println!("Read array: {:?}", read_array),
        Err(e) => eprintln!("Errore nella lettura: {:?}", e),
    }

    // Pulizia esplicita di dbshm prima che esca dallo scope naturale
    mem::drop(dbshm);
    println!("dbshm è stato eliminato");
    // dbshm è stato eliminato, quindi non è più accessibile da qui in poi
    // Puoi eseguire altre operazioni o pulizie prima che il programma termini
}
