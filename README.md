
# DoubleBufferedSharedMemory Library

## Overview
The `DoubleBufferedSharedMemory` library provides a robust and efficient way to manage shared memory with double buffering. This Rust library is designed to facilitate synchronized read and write operations to shared memory, making it ideal for high-performance, concurrent applications. 

## Features
- **Generic Implementation**: Works with any data type that implements Serialize, Deserialize, Any, Zero, and Clone.
- **Double Buffering**: Minimizes waiting time by allowing operations to proceed on one buffer while the other is being used.
- **Synchronized Access**: Ensures that read and write operations are thread-safe and do not interfere with each other.
- **Error Handling**: Provides comprehensive error information through the `DbShmError` type.

## Usage

### Creating a New Instance
To create a new instance of `DoubleBufferedSharedMemory`, you need to specify the base name for shared buffers and the shape of the data.

```rust
let dbshm = DoubleBufferedSharedMemory::<u8>::new("my_shared_buffer", (100, 100, 3));
```

### Writing to the Active Buffer
To write data to the active buffer, use the `write` method with an `ArrayD<T>` reference. Ensure that the data type `T` matches the type specified for the `DoubleBufferedSharedMemory`.

```rust
let array = ArrayD::<u8>::zeros(IxDyn(&[100, 100, 3]));
dbshm.write(&array).expect("Failed to write to shared memory");
```

### Reading from the Inactive Buffer
To read data from the inactive buffer, use the `read` method. It returns an `ArrayD<T>` containing the deserialized data.

```rust
let read_array = dbshm.read().expect("Failed to read from shared memory");
```

### Getting Memory Size
To retrieve the total size of the allocated shared memory buffer, use the `get_memory_size` method.

```rust
let memory_size = dbshm.get_memory_size();
println!("The size of the shared memory buffer is {} bytes.", memory_size);
```

## Error Handling
The library uses `DbShmError` to represent various errors that can occur during operations. Handle these errors appropriately in your application.

## Contribution
Contributions are welcome. Feel free to fork, modify, and submit pull requests.

## License
Specify your license or indicate that the project is unlicensed.
