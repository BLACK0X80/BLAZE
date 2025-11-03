# BLAZE Cookbook

Practical recipes for common programming tasks in BLAZE.

---

## Table of Contents

1. [File Operations](#file-operations)
2. [Networking](#networking)
3. [Concurrency](#concurrency)
4. [Error Handling](#error-handling)
5. [Collections](#collections)
6. [Pattern Matching](#pattern-matching)
7. [Traits and Generics](#traits-and-generics)

---

## File Operations

### Reading a Text File

```blaze
use std::io::{File, IoError};
use std::result::Result;

fn read_config() -> Result<String, IoError> {
    let mut file = File::open("config.txt")?;
    let mut buffer = [0u8; 1024];
    let bytes_read = file.read(&mut buffer)?;
    
    String::from_utf8(Vec::from(&buffer[..bytes_read]))
        .map_err(|_| IoError::new("Invalid UTF-8"))
}
```

### Writing to a File

```blaze
fn write_log(message: &str) -> Result<(), IoError> {
    let mut file = File::create("app.log")?;
    file.write(message.as_bytes())?;
    file.write(b"\n")?;
    Ok(())
}
```

### Processing Lines

```blaze
fn count_lines(path: &str) -> Result<usize, IoError> {
    let content = read_file_to_string(path)?;
    Ok(content.lines().count())
}
```

---

## Networking

### Simple HTTP GET Request

```blaze
async fn fetch_data(url: &str) -> Result<String, IoError> {
    let mut stream = TcpStream::connect("example.com:80").await?;
    
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n",
        url
    );
    
    stream.write(request.as_bytes()).await?;
    
    let mut response = Vec::new();
    let mut buffer = [0u8; 4096];
    
    loop {
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 { break; }
        response.extend_from_slice(&buffer[..bytes_read]);
    }
    
    String::from_utf8(response).map_err(|_| IoError::new("Invalid response"))
}
```

### Echo Server

```blaze
async fn run_echo_server() -> Result<(), IoError> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println("Echo server listening on port 8080");
    
    loop {
        let (mut stream, addr) = listener.accept().await?;
        println("New connection from {}", addr);
        
        spawn(async move {
            let mut buffer = [0u8; 1024];
            
            loop {
                match stream.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        stream.write(&buffer[..n]).await.ok();
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
```

### REST API Client

```blaze
struct ApiClient {
    base_url: String,
}

impl ApiClient {
    fn new(base_url: String) -> Self {
        Self { base_url }
    }
    
    async fn get_user(&self, id: i32) -> Result<User, ApiError> {
        let url = format!("{}/users/{}", self.base_url, id);
        let response = self.request(&url).await?;
        parse_json::<User>(&response)
    }
    
    async fn request(&self, url: &str) -> Result<String, ApiError> {
        // HTTP request implementation
    }
}
```

---

## Concurrency

### Parallel Processing

```blaze
use std::async_rt::{spawn, JoinHandle};

async fn process_items(items: Vec<Item>) -> Vec<Result<Output, Error>> {
    let mut handles = Vec::new();
    
    for item in items {
        let handle = spawn(async move {
            process_single_item(item).await
        });
        handles.push(handle);
    }
    
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }
    
    results
}

async fn process_single_item(item: Item) -> Result<Output, Error> {
    // Process item
}
```

### Shared State with Mutex

```blaze
use std::async_rt::Mutex;

struct Counter {
    value: Mutex<i32>,
}

impl Counter {
    fn new() -> Self {
        Self {
            value: Mutex::new(0),
        }
    }
    
    async fn increment(&self) {
        let mut guard = self.value.lock().await;
        *guard += 1;
    }
    
    async fn get(&self) -> i32 {
        let guard = self.value.lock().await;
        *guard
    }
}
```

### Worker Pool

```blaze
struct WorkerPool {
    workers: Vec<Worker>,
    task_queue: Mutex<Vec<Task>>,
}

impl WorkerPool {
    async fn new(num_workers: usize) -> Self {
        let mut workers = Vec::new();
        let task_queue = Mutex::new(Vec::new());
        
        for id in 0..num_workers {
            workers.push(Worker::new(id));
        }
        
        Self { workers, task_queue }
    }
    
    async fn submit(&self, task: Task) {
        let mut queue = self.task_queue.lock().await;
        queue.push(task);
    }
    
    async fn run(&self) {
        for worker in &self.workers {
            spawn(async move {
                worker.process_tasks().await;
            });
        }
    }
}
```

---

## Error Handling

### Custom Error Types

```blaze
enum AppError {
    Io(IoError),
    Parse(String),
    Network(String),
    NotFound,
}

impl AppError {
    fn message(&self) -> String {
        match self {
            AppError::Io(e) => format!("IO error: {}", e.message()),
            AppError::Parse(msg) => format!("Parse error: {}", msg),
            AppError::Network(msg) => format!("Network error: {}", msg),
            AppError::NotFound => String::from("Resource not found"),
        }
    }
}
```

### Error Propagation

```blaze
fn load_config() -> Result<Config, AppError> {
    let content = read_file("config.toml")
        .map_err(|e| AppError::Io(e))?;
    
    parse_toml(&content)
        .map_err(|e| AppError::Parse(e))
}
```

### Retry Logic

```blaze
async fn retry<F, T, E>(mut f: F, max_attempts: usize) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    for attempt in 1..=max_attempts {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_attempts => return Err(e),
            Err(_) => {
                sleep(1000 * attempt as u64).await;
            }
        }
    }
    
    unreachable!()
}
```

---

## Collections

### Building a HashMap

```blaze
use std::collections::HashMap;

fn word_frequency(text: &str) -> HashMap<String, usize> {
    let mut freq = HashMap::new();
    
    for word in text.split_whitespace() {
        let count = freq.get(&word).unwrap_or(&0);
        freq.insert(word.to_string(), count + 1);
    }
    
    freq
}
```

### Filtering and Mapping

```blaze
fn process_numbers(numbers: Vec<i32>) -> Vec<i32> {
    numbers
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
        .collect()
}
```

### Custom Iterator

```blaze
struct RangeIterator {
    current: i32,
    end: i32,
}

impl Iterator for RangeIterator {
    type Item = i32;
    
    fn next(&mut self) -> Option<i32> {
        if self.current < self.end {
            let value = self.current;
            self.current += 1;
            Some(value)
        } else {
            None
        }
    }
}
```

---

## Pattern Matching

### Enum Matching

```blaze
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

fn process_message(msg: Message) {
    match msg {
        Message::Quit => {
            println("Quitting...");
        }
        Message::Move { x, y } => {
            println("Moving to ({}, {})", x, y);
        }
        Message::Write(text) => {
            println("Writing: {}", text);
        }
        Message::ChangeColor(r, g, b) => {
            println("Color: rgb({}, {}, {})", r, g, b);
        }
    }
}
```

### Guards

```blaze
fn classify_number(n: i32) -> String {
    match n {
        x if x < 0 => String::from("negative"),
        0 => String::from("zero"),
        x if x < 10 => String::from("small positive"),
        x if x < 100 => String::from("medium positive"),
        _ => String::from("large positive"),
    }
}
```

### Destructuring

```blaze
struct Point { x: i32, y: i32 }

fn describe_point(point: Point) -> String {
    match point {
        Point { x: 0, y: 0 } => String::from("origin"),
        Point { x: 0, y } => format!("on y-axis at {}", y),
        Point { x, y: 0 } => format!("on x-axis at {}", x),
        Point { x, y } => format!("at ({}, {})", x, y),
    }
}
```

---

## Traits and Generics

### Generic Function

```blaze
fn find_max<T: PartialOrd>(items: &[T]) -> Option<&T> {
    if items.is_empty() {
        return None;
    }
    
    let mut max = &items[0];
    for item in items.iter().skip(1) {
        if item > max {
            max = item;
        }
    }
    
    Some(max)
}
```

### Trait Implementation

```blaze
trait Drawable {
    fn draw(&self);
}

struct Circle {
    radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println("Drawing circle with radius {}", self.radius);
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println("Drawing rectangle {}x{}", self.width, self.height);
    }
}

fn render_all(shapes: &[Box<dyn Drawable>]) {
    for shape in shapes {
        shape.draw();
    }
}
```

### Builder Pattern

```blaze
struct HttpRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

struct HttpRequestBuilder {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpRequestBuilder {
    fn new(method: String, url: String) -> Self {
        Self {
            method,
            url,
            headers: HashMap::new(),
            body: None,
        }
    }
    
    fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
    
    fn build(self) -> HttpRequest {
        HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body,
        }
    }
}

// Usage
let request = HttpRequestBuilder::new(String::from("POST"), String::from("/api/users"))
    .header(String::from("Content-Type"), String::from("application/json"))
    .body(String::from("{\"name\": \"Alice\"}"))
    .build();
```

---

## Advanced Patterns

### State Machine

```blaze
enum State {
    Idle,
    Processing { progress: f64 },
    Completed { result: String },
    Failed { error: String },
}

struct StateMachine {
    state: State,
}

impl StateMachine {
    fn new() -> Self {
        Self { state: State::Idle }
    }
    
    fn start(&mut self) {
        match self.state {
            State::Idle => {
                self.state = State::Processing { progress: 0.0 };
            }
            _ => {
                println("Already started");
            }
        }
    }
    
    fn update(&mut self, progress: f64) {
        match self.state {
            State::Processing { .. } => {
                if progress >= 1.0 {
                    self.state = State::Completed {
                        result: String::from("Done"),
                    };
                } else {
                    self.state = State::Processing { progress };
                }
            }
            _ => {}
        }
    }
}
```

---

**More recipes coming soon!** Contribute your own at https://github.com/BLACK0X80/BLAZE
