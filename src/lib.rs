extern crate shared_memory;
pub mod errors;
use shared_memory::*;
use std::sync::{Arc, Mutex, Condvar};
use errors::DbShmError;
use std::any::Any;
use std::marker::PhantomData;
use ndarray::{ArrayD, IxDyn};
use bincode;
use serde::{Serialize, Deserialize};
use std::mem;
use num_traits::Zero;
use std::error::Error;

/// Structure for managing shared memory with double buffering.
/// This structure is generic over `T` which must implement Serialize, Deserialize, Any, Zero, and Clone.
/// It provides synchronized read and write operations to a shared memory space with double buffering to minimize waiting time.
pub struct DoubleBufferedSharedMemory<T> {
    // Internal shared memory buffers
    buffers: Vec<shared_memory::Shmem>,
    // Index of the currently active buffer for writing
    active_index: usize,
    // Total size of each buffer
    size: usize,
    // Permits for synchronized read access
    read_permits: Arc<(Mutex<usize>, Condvar)>,
    // Permits for synchronized write access
    write_permits: Arc<(Mutex<bool>, Condvar)>,
    // PhantomData to associate generic type T with the struct without storing it
    _phantom: PhantomData<T>,
}


impl<T> DoubleBufferedSharedMemory<T>
where
    T: Serialize + Deserialize<'static> + Any + Zero + Clone,
{
    /// Creates a new instance of `DoubleBufferedSharedMemory`.
    ///
    /// # Parameters
    ///
    /// * `name_base`: The base name to identify shared buffers. Each buffer will be identified as "{name_base}_0" and "{name_base}_1".
    /// * `shape`: A tuple representing the dimensions (height, width, channels) of the data. This is used to calculate the necessary buffer size.
    ///
    /// # Return
    ///
    /// Returns a `Result` containing an instance of `DoubleBufferedSharedMemory` if initialization is successful,
    /// or a `ShmemError` error otherwise. The error may occur due to issues in creating shared memory segments.
    ///
    /// # Examples
    ///
    /// 
    /// let dbshm = DoubleBufferedSharedMemory::<u8>::new("my_shared_buffer", (100, 100, 3));
    /// 
    pub fn new(name_base: &str, shape: (usize, usize, usize)) -> Result<Self, Box<dyn Error>> {
        let dtype_size: usize = mem::size_of::<T>();
        let base_size = shape.0 * shape.1 * shape.2 * dtype_size;

        let extra_size = Self::calc_extra_size(base_size, shape)?;
        let size = base_size + extra_size; // Dimensione totale necessaria
        let mut buffers = Vec::new();

        for i in 0..2 {
            let name = format!("{}_{}", name_base, i);
            let shm = ShmemConf::new()
                .size(size)
                .os_id(&name)
                .create()?;
            buffers.push(shm);
        }
        Ok(Self {
            buffers,
            active_index: 0,
            size,
            read_permits: Arc::new((Mutex::new(1), Condvar::new())), // Permette una lettura alla volta
            write_permits: Arc::new((Mutex::new(true), Condvar::new())), // Permette una scrittura alla volta
            _phantom: PhantomData,
        })
    }


    fn calc_extra_size(base_size: usize, shape: (usize, usize, usize)) -> Result<usize, DbShmError> {
        let shape_dyn = IxDyn(&[shape.0, shape.1, shape.2]);
        // Calcola la dimensione necessaria per la serializzazione di un piccolo campione di dati
        let sample_data = ArrayD::<T>::zeros(shape_dyn); // Crea un array di zeri
        let serialized_sample = bincode::serialize(&sample_data)
        .map_err(|e| DbShmError::SerializationError(e.to_string(), base_size, sample_data.len()))?;
        let extra_size = serialized_sample.len() - base_size; // Calcola lo spazio extra necessario
        Ok(extra_size)
    }
    /// Acquires the write permit. This function blocks the thread until the write permit is available.
    fn acquire_write_permit(&self) {
        let (lock, cvar) = &*self.write_permits;
        let mut permit = lock.lock().unwrap();
        while !*permit {
            permit = cvar.wait(permit).unwrap();
        }
        *permit = false;
    }

    /// Releases the write permit.
    fn release_write_permit(&self) {
        let (lock, cvar) = &*self.write_permits;
        let mut permit = lock.lock().unwrap();
        *permit = true;
        cvar.notify_all();
    }

    /// Acquires the read permit. This function blocks the thread until the read permit is available.
    fn acquire_read_permit(&self) {
        let (lock, cvar) = &*self.read_permits;
        let mut permit = lock.lock().unwrap();
        while *permit == 0 {
            permit = cvar.wait(permit).unwrap();
        }
        *permit -= 1;
    }

    /// Releases the read permit.
    fn release_read_permit(&self) {
        let (lock, cvar) = &*self.read_permits;
        let mut permit = lock.lock().unwrap();
        *permit += 1;
        cvar.notify_all();
    }

    
    /// Writes data to the active buffer.
    ///
    /// # Parameters
    ///
    /// * `array`: A reference to an `ArrayD<T>` array to be written to the active buffer. The data type `T` must match the type specified for the `DoubleBufferedSharedMemory`.
    ///
    /// # Return
    ///
    /// Returns a `Result` with an empty value `Ok(())` if the write is successful,
    /// or a `DbShmError` error in case of problems during writing. The error might be due to serialization issues or size mismatches.
    ///
    /// # Examples
    ///
    /// 
    /// let mut dbshm = DoubleBufferedSharedMemory::<u8>::new("my_shared_buffer", (100, 100, 3)).unwrap();
    /// let array = ArrayD::<u8>::zeros(IxDyn(&[100, 100, 3]));
    /// dbshm.write(&array).expect("Failed to write to shared memory");
    /// 
    pub fn write(&mut self, array: &ArrayD<T>) -> Result<(), DbShmError> {
        self.acquire_write_permit();
        let data = bincode::serialize(array)
        .map_err(|e| DbShmError::SerializationError(e.to_string(), self.size, array.len()))?;

        if data.len() != self.size {
            self.release_write_permit();
            return Err(DbShmError::InvalidSize(self.size, data.len()));
        }
        
        let active_buffer = &mut self.buffers[self.active_index];
        let buffer_slice = unsafe {
            // Ottieni un riferimento mutable ai tuoi dati attivi
            std::slice::from_raw_parts_mut(active_buffer.as_ptr() as *mut u8, self.size)
        };
    
        // Copia gli ultimi self.size bytes da data a buffer_slice
        buffer_slice.copy_from_slice(&data);

        self.active_index = 1 - self.active_index; // Cambia il buffer attivo
        self.release_write_permit();
        Ok(())

    }

    /// Reads data from the inactive buffer.
    ///
    /// This method deserializes and returns the data from the currently inactive buffer.
    /// It ensures that only one read operation can occur at a time through the use of read permits.
    ///
    /// # Return
    ///
    /// Returns a `Result` containing an `ArrayD<T>` array if the read is successful,
    /// or a `DbShmError` in case of problems during reading.
    pub fn read(&self) -> Result<ArrayD<T>, DbShmError> {
        self.acquire_read_permit();

        let read_index = 1 - self.active_index;
        let inactive_buffer = &self.buffers[read_index];
        // let start_index = inactive_buffer.len().wrapping_sub(self.size);

        
        // Ottieni una slice che inizia da start_index e si estende per self.size bytes.
        let data = unsafe {
            std::slice::from_raw_parts(inactive_buffer.as_ptr() as *const u8, self.size)
        };
        let deserialized_data = bincode::deserialize(data)
        .map_err(|e| DbShmError::SerializationError(e.to_string(), self.size, data.len()))?;
        self.release_read_permit();
        Ok(deserialized_data)
    }

    /// Returns the size of the shared memory buffer.
    ///
    /// This method retrieves the total size in bytes of the allocated shared memory buffer
    /// used by the instance of `DoubleBufferedSharedMemory`. This size is determined during
    /// the creation of the `DoubleBufferedSharedMemory` instance and remains constant
    /// throughout its lifetime.
    ///
    /// # Examples
    ///
    /// 
    /// Suppose `dbshm` is an instance of `DoubleBufferedSharedMemory`.
    /// let memory_size = dbshm.get_memory_size();
    /// println!("The size of the shared memory buffer is {} bytes.", memory_size);
    /// 
    ///
    /// # Return
    ///
    /// Returns the size of the shared memory buffer in bytes as a `usize`.
    pub fn get_memory_size(&self) -> usize {
        self.size
    }

    /// Releases the resources associated with the shared memory.
    ///
    /// This method is called automatically when an instance of `DoubleBufferedSharedMemory` goes
    /// out of scope. It releases the resources by draining and dropping all the shared memory buffers.
    /// It's part of Rust's `Drop` trait, which provides a way to run some code when a value goes out of scope.
    ///
    /// This is where you should put all the necessary cleanup code that should be executed when
    /// your instance is about to be destroyed.
    ///
    /// Note: It's not common or recommended to call this method directly. It's automatically
    /// invoked when an instance of the struct goes out of scope.
    ///
    /// # Examples
    ///
    /// 
    /// {
    ///     let dbshm = DoubleBufferedSharedMemory::<u8>::new("my_shared_mem", (1080, 1920, 3));
    ///  Use `dbshm`...
    /// }  
    /// `drop` is called automatically here
    /// 
    pub fn drop(&mut self) {
        // Qui andremo a rilasciare le risorse allocate, in particolare la memoria condivisa.
        for _ in self.buffers.drain(..) {
            // Usa std::mem::drop per rilasciare esplicitamente la memoria condivisa.
            // In molti casi, questo non è strettamente necessario poiché Rust rilascia automaticamente
            // le risorse quando un oggetto esce dallo scope, ma lo includiamo qui per completezza
            // e per esprimere esplicitamente l'intenzione di rilasciare la risorsa.
            
        }
    }

}