# BLAZE Standard Library API Reference

Complete API documentation for the BLAZE standard library.

---

## Core Types

### `Option<T>`

**Definition**:
```blaze
pub enum Option<T> {
    Some(T),
    None,
}
```

**Methods**:

#### `is_some() -> bool`
Returns `true` if the option contains a value.

```blaze
let x = Some(5);
assert!(x.is_some());
```

#### `is_none() -> bool`
Returns `true` if the option is `None`.

#### `unwrap() -> T`
Returns the contained value, panics if `None`.

```blaze
let x = Some("value");
assert_eq!(x.unwrap(), "value");
```

#### `unwrap_or(default: T) -> T`
Returns the contained value or a default.

```blaze
let x: Option<i32> = None;
assert_eq!(x.unwrap_or(10), 10);
```

#### `map<U, F>(f: F) -> Option<U>` where `F: FnOnce(T) -> U`
Maps an `Option<T>` to `Option<U>` by applying a function.

```blaze
let x = Some(5);
let y = x.map(|n| n * 2);
assert_eq!(y, Some(10));
```

#### `and_then<U, F>(f: F) -> Option<U>` where `F: FnOnce(T) -> Option<U>`
Chains optional computations.

```blaze
fn divide(x: i32, y: i32) -> Option<i32> {
    if y == 0 { None } else { Some(x / y) }
}

let result = Some(10).and_then(|x| divide(x, 2));
assert_eq!(result, Some(5));
```

---

### `Result<T, E>`

**Definition**:
```blaze
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

**Methods**:

#### `is_ok() -> bool`
Returns `true` if the result is `Ok`.

#### `is_err() -> bool`
Returns `true` if the result is `Err`.

#### `unwrap() -> T`
Returns the contained `Ok` value, panics on `Err`.

#### `unwrap_err() -> E`
Returns the contained `Err` value, panics on `Ok`.

#### `expect(msg: &str) -> T`
Returns the contained value or panics with a custom message.

```blaze
let x: Result<i32, &str> = Err("emergency");
x.expect("Testing expect"); // panics with "Testing expect: emergency"
```

#### `map<U, F>(f: F) -> Result<U, E>` where `F: FnOnce(T) -> U`
Maps a `Result<T, E>` to `Result<U, E>`.

#### `and_then<U, F>(f: F) -> Result<U, E>` where `F: FnOnce(T) -> Result<U, E>`
Chains result-returning operations.

```blaze
fn parse_int(s: &str) -> Result<i32, String> {
    // ... parsing logic
}

let result = Ok("42").and_then(parse_int);
```

---

### `String`

**Definition**:
```blaze
pub struct String {
    bytes: Vec<u8>,
}
```

**Methods**:

#### `new() -> String`
Creates a new empty string.

#### `with_capacity(capacity: usize) -> String`
Creates a string with the specified capacity.

#### `from(s: &str) -> String`
Creates a string from a string slice.

#### `len() -> usize`
Returns the length in bytes.

#### `is_empty() -> bool`
Returns `true` if the string is empty.

#### `push(ch: char)`
Appends a character.

```blaze
let mut s = String::new();
s.push('h');
s.push('i');
assert_eq!(s, "hi");
```

#### `push_str(s: &str)`
Appends a string slice.

#### `pop() -> Option<char>`
Removes and returns the last character.

#### `split(delimiter: char) -> Split`
Returns an iterator over substrings.

```blaze
let s = String::from("a,b,c");
for part in s.split(',') {
    println("{}", part);
}
```

#### `trim() -> &str`
Returns a string slice with whitespace removed.

#### `to_lowercase() -> String`
Returns a lowercase version.

#### `to_uppercase() -> String`
Returns an uppercase version.

---

### `Vec<T>`

**Definition**:
```blaze
pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}
```

**Methods**:

#### `new() -> Vec<T>`
Creates a new empty vector.

#### `with_capacity(capacity: usize) -> Vec<T>`
Creates a vector with the specified capacity.

#### `len() -> usize`
Returns the number of elements.

#### `is_empty() -> bool`
Returns `true` if empty.

#### `push(value: T)`
Appends an element.

```blaze
let mut v = Vec::new();
v.push(1);
v.push(2);
v.push(3);
```

#### `pop() -> Option<T>`
Removes and returns the last element.

#### `insert(index: usize, value: T)`
Inserts an element at the specified index.

#### `remove(index: usize) -> T`
Removes and returns the element at the specified index.

#### `get(index: usize) -> Option<&T>`
Returns a reference to the element at the index.

#### `iter() -> VecIter<T>`
Returns an iterator.

```blaze
let v = vec![1, 2, 3];
for item in v.iter() {
    println("{}", item);
}
```

---

## Collections

### `HashMap<K, V>`

**Definition**:
```blaze
pub struct HashMap<K, V> {
    buckets: Vec<Vec<Entry<K, V>>>,
    len: usize,
}
```

**Methods**:

#### `new() -> HashMap<K, V>`
Creates a new empty hash map.

#### `with_capacity(capacity: usize) -> HashMap<K, V>`
Creates a hash map with the specified capacity.

#### `insert(key: K, value: V) -> Option<V>`
Inserts a key-value pair.

```blaze
let mut map = HashMap::new();
map.insert("key", 42);
```

#### `get(key: &K) -> Option<&V>`
Returns a reference to the value.

```blaze
let value = map.get("key");
```

#### `remove(key: &K) -> Option<V>`
Removes and returns the value.

#### `contains_key(key: &K) -> bool`
Returns `true` if the key exists.

#### `iter() -> HashMapIter<K, V>`
Returns an iterator over key-value pairs.

---

## I/O

### Functions

#### `print(s: &str)`
Prints to stdout without newline.

#### `println(s: &str)`
Prints to stdout with newline.

```blaze
println("Hello, World!");
```

#### `eprint(s: &str)`
Prints to stderr without newline.

#### `eprintln(s: &str)`
Prints to stderr with newline.

#### `read_line() -> Result<String, IoError>`
Reads a line from stdin.

```blaze
let line = read_line()?;
println("You entered: {}", line);
```

### `File`

#### `open(path: &str) -> Result<File, IoError>`
Opens a file for reading.

#### `create(path: &str) -> Result<File, IoError>`
Creates a new file.

#### `read(&mut self, buffer: &mut [u8]) -> Result<usize, IoError>`
Reads bytes into the buffer.

#### `write(&mut self, data: &[u8]) -> Result<usize, IoError>`
Writes bytes to the file.

```blaze
let mut file = File::create("output.txt")?;
file.write(b"Hello, file!")?;
```

---

## Async Runtime

### `spawn<F>(future: F)` where `F: Future<Output = ()>`
Spawns an async task.

```blaze
spawn(async {
    println("Running in async task");
});
```

### `sleep(duration_ms: u64)`
Sleeps for the specified duration.

```blaze
async fn delayed_print() {
    sleep(1000).await;
    println("One second later");
}
```

### `Mutex<T>`

Thread-safe mutual exclusion primitive.

#### `new(data: T) -> Mutex<T>`
Creates a new mutex.

#### `lock() -> MutexGuard<T>`
Acquires the lock.

```blaze
let mutex = Mutex::new(0);
let mut guard = mutex.lock().await;
*guard += 1;
```

---

## Networking

### `TcpListener`

#### `bind(addr: &str) -> Result<TcpListener, IoError>`
Binds to an address.

```blaze
let listener = TcpListener::bind("127.0.0.1:8080")?;
```

#### `accept() -> Result<(TcpStream, String), IoError>`
Accepts a new connection.

```blaze
let (stream, addr) = listener.accept().await?;
```

### `TcpStream`

#### `connect(addr: &str) -> Result<TcpStream, IoError>`
Connects to a server.

#### `read(&mut self, buffer: &mut [u8]) -> Result<usize, IoError>`
Reads data.

#### `write(&mut self, data: &[u8]) -> Result<usize, IoError>`
Writes data.

---

## Traits

### `Clone`

```blaze
pub trait Clone {
    fn clone(&self) -> Self;
}
```

### `PartialEq`

```blaze
pub trait PartialEq {
    fn eq(&self, other: &Self) -> bool;
}
```

### `Iterator`

```blaze
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

---

## Macros

### `println!`
Prints formatted text with newline.

```blaze
println!("Value: {}", 42);
println!("x={}, y={}", x, y);
```

### `vec!`
Creates a vector.

```blaze
let v = vec![1, 2, 3, 4, 5];
```

### `assert!`
Panics if condition is false.

```blaze
assert!(x > 0);
assert_eq!(x, 5);
```

---

For more examples, see `EXAMPLES_GUIDE.md`.
