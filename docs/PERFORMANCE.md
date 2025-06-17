# Performance Optimization Guide 🚀

This document provides comprehensive guidance for optimizing WhatsApp Engine Rust performance across different deployment scenarios and use cases.

## 📋 Table of Contents

- [Performance Overview](#-performance-overview)
- [Benchmarking and Profiling](#-benchmarking-and-profiling)
- [CPU Optimization](#-cpu-optimization)
- [Memory Optimization](#-memory-optimization)
- [Network Optimization](#-network-optimization)
- [Browser Performance](#-browser-performance)
- [Database and Storage](#-database-and-storage)
- [Concurrency and Async](#-concurrency-and-async)
- [Caching Strategies](#-caching-strategies)
- [Resource Management](#-resource-management)
- [Production Optimization](#-production-optimization)
- [Monitoring Performance](#-monitoring-performance)

## 🎯 Performance Overview

### Performance Goals

| Metric | Target | Acceptable | Poor |
|--------|--------|------------|------|
| **API Response Time** | <200ms | <500ms | >1s |
| **Message Send Time** | <2s | <5s | >10s |
| **Authentication Time** | <30s | <60s | >120s |
| **Memory Usage** | <512MB | <1GB | >2GB |
| **CPU Usage** | <50% | <80% | >90% |
| **Concurrent Users** | 100+ | 50+ | <20 |
| **Messages/Second** | 10+ | 5+ | <2 |

### Performance Characteristics

```rust
// Performance benchmarking framework
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;

fn benchmark_message_sending(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let engine = rt.block_on(async {
        WhatsAppEngine::new_test().await.unwrap()
    });
    
    let mut group = c.benchmark_group("message_sending");
    
    for message_size in [10, 100, 1000, 10000].iter() {
        let message = "x".repeat(*message_size);
        
        group.bench_with_input(
            BenchmarkId::new("send_message", message_size),
            &message,
            |b, msg| {
                b.to_async(&rt).iter(|| async {
                    black_box(
                        engine.send_message("+1234567890", msg).await
                    )
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_message_sending);
criterion_main!(benches);
```

## 📊 Benchmarking and Profiling

### Cargo Benchmarks

```toml
# Cargo.toml
[[bench]]
name = "performance"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

```rust
// benches/performance.rs
use criterion::{criterion_group, criterion_main, Criterion};
use whatsapp_engine::*;

fn engine_creation_benchmark(c: &mut Criterion) {
    c.bench_function("engine_creation", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let engine = WhatsAppEngine::with_defaults().await.unwrap();
                engine.close().await.unwrap();
            })
    });
}

fn authentication_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = rt.block_on(async {
        WhatsAppEngine::new_test().await.unwrap()
    });
    
    c.bench_function("qr_generation", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = engine.authenticate_with_qr().await;
        })
    });
}

criterion_group!(
    benches,
    engine_creation_benchmark,
    authentication_benchmark
);
criterion_main!(benches);
```

### Profiling Tools

#### CPU Profiling with `perf`

```bash
# Install perf
sudo apt-get install linux-tools-common linux-tools-generic

# Profile application
sudo perf record -g --call-graph dwarf target/release/whatsapp-server
sudo perf report

# Generate flame graph
sudo perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

#### Memory Profiling with `valgrind`

```bash
# Install valgrind
sudo apt-get install valgrind

# Memory leak detection
valgrind --tool=memcheck --leak-check=full target/release/whatsapp-server

# Memory usage profiling
valgrind --tool=massif target/release/whatsapp-server
```

#### Rust-specific Profiling

```rust
// src/utils/profiling.rs
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct Profiler {
    start_times: HashMap<String, Instant>,
    durations: HashMap<String, Vec<Duration>>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            start_times: HashMap::new(),
            durations: HashMap::new(),
        }
    }
    
    pub fn start_timing(&mut self, operation: &str) {
        self.start_times.insert(operation.to_string(), Instant::now());
    }
    
    pub fn end_timing(&mut self, operation: &str) {
        if let Some(start_time) = self.start_times.remove(operation) {
            let duration = start_time.elapsed();
            self.durations
                .entry(operation.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }
    
    pub fn get_stats(&self, operation: &str) -> Option<ProfileStats> {
        self.durations.get(operation).map(|durations| {
            let total: Duration = durations.iter().sum();
            let count = durations.len();
            let avg = total / count as u32;
            let min = *durations.iter().min().unwrap();
            let max = *durations.iter().max().unwrap();
            
            ProfileStats { total, avg, min, max, count }
        })
    }
}

#[derive(Debug)]
pub struct ProfileStats {
    pub total: Duration,
    pub avg: Duration,
    pub min: Duration,
    pub max: Duration,
    pub count: usize,
}
```

## ⚡ CPU Optimization

### Async/Await Optimization

```rust
// Optimize async task spawning
use tokio::task::JoinSet;

impl ChatService {
    pub async fn send_bulk_messages_optimized(
        &self,
        messages: Vec<(String, String)>,
    ) -> Result<Vec<SendMessageResult>> {
        const MAX_CONCURRENT: usize = 10;
        let mut set = JoinSet::new();
        let mut results = Vec::with_capacity(messages.len());
        
        for chunk in messages.chunks(MAX_CONCURRENT) {
            for (phone, message) in chunk {
                let phone = phone.clone();
                let message = message.clone();
                let service = self.clone();
                
                set.spawn(async move {
                    service.send_message(&phone, &message).await
                });
            }
            
            // Wait for current batch to complete
            while let Some(result) = set.join_next().await {
                results.push(result??);
            }
        }
        
        Ok(results)
    }
}
```

### CPU-bound Task Optimization

```rust
// Use rayon for CPU-intensive work
use rayon::prelude::*;

impl MessageProcessor {
    pub fn process_messages_parallel(&self, messages: Vec<Message>) -> Vec<ProcessedMessage> {
        messages
            .into_par_iter()
            .map(|msg| self.process_single_message(msg))
            .collect()
    }
    
    pub fn validate_phones_parallel(&self, phones: Vec<String>) -> Vec<ValidationResult> {
        phones
            .par_iter()
            .map(|phone| self.validate_phone_number(phone))
            .collect()
    }
}

// Move blocking operations to blocking thread pool
use tokio::task;

pub async fn cpu_intensive_operation(data: Vec<u8>) -> Result<ProcessedData> {
    task::spawn_blocking(move || {
        // CPU-intensive work here
        expensive_computation(data)
    }).await?
}
```

### Compilation Optimizations

```toml
# Cargo.toml
[profile.release]
lto = true                  # Link-time optimization
codegen-units = 1          # Better optimization
panic = "abort"            # Smaller binary size
strip = true               # Remove debug symbols

[profile.release-with-debug]
inherits = "release"
debug = true               # Keep debug info for profiling

# Target-specific optimizations
[profile.release.package."*"]
opt-level = 3

# Platform-specific optimizations
[target.'cfg(target_arch = "x86_64")']
rustflags = ["-C", "target-cpu=native"]
```

## 🧠 Memory Optimization

### Memory Pool Management

```rust
// Custom allocator for better memory management
use std::alloc::{GlobalAlloc, Layout};
use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Object pooling for frequently allocated objects
use object_pool::{Pool, Reusable};

pub struct MessagePool {
    pool: Pool<Message>,
}

impl MessagePool {
    pub fn new() -> Self {
        Self {
            pool: Pool::new(100, || Message::default()),
        }
    }
    
    pub fn get_message(&self) -> Reusable<Message> {
        self.pool.try_pull().unwrap_or_else(|| {
            self.pool.pull(Message::default())
        })
    }
}
```

### Efficient Data Structures

```rust
// Use Box<str> instead of String for immutable strings
pub struct Contact {
    pub id: Box<str>,
    pub name: Box<str>, 
    pub phone: Box<str>,
    pub is_business: bool,
}

// Use SmallVec for collections that are usually small
use smallvec::{SmallVec, smallvec};

pub struct MessageBatch {
    pub messages: SmallVec<[Message; 8]>, // Stack allocation for ≤8 items
}

// Use Cow for potentially borrowed data
use std::borrow::Cow;

pub fn process_phone_number(phone: Cow<str>) -> String {
    match phone {
        Cow::Borrowed(s) if s.starts_with('+') => s.to_string(),
        Cow::Borrowed(s) => format!("+{}", s),
        Cow::Owned(s) if s.starts_with('+') => s,
        Cow::Owned(s) => format!("+{}", s),
    }
}
```

### Memory Leak Prevention

```rust
// Weak references to prevent cycles
use std::sync::{Arc, Weak};

pub struct AuthService {
    browser: Arc<BrowserService>,
    engine: Weak<WhatsAppEngine>, // Weak reference to parent
}

// Explicit cleanup for resources
impl Drop for WhatsAppEngine {
    fn drop(&mut self) {
        // Explicit cleanup
        if let Some(browser) = &self.browser_service {
            browser.shutdown();
        }
    }
}

// Memory-conscious caching
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct ContactCache {
    cache: LruCache<String, Contact>,
}

impl ContactCache {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
        }
    }
    
    pub fn get_or_insert(&mut self, id: &str, fetcher: impl FnOnce() -> Contact) -> &Contact {
        if !self.cache.contains(id) {
            let contact = fetcher();
            self.cache.put(id.to_string(), contact);
        }
        self.cache.get(id).unwrap()
    }
}
```

## 🌐 Network Optimization

### Connection Pooling

```rust
// HTTP client with connection pooling
use reqwest::Client;
use std::time::Duration;

pub fn create_optimized_http_client() -> Client {
    Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .tcp_keepalive(Duration::from_secs(60))
        .tcp_nodelay(true)
        .build()
        .unwrap()
}

// WebSocket connection optimization
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;

pub async fn create_optimized_websocket(
    url: &str
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let mut request = url.into_client_request()?;
    
    // Add performance headers
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        "whatsapp".parse().unwrap()
    );
    
    let (ws_stream, _) = tokio_tungstenite::client_async(request, url).await?;
    Ok(ws_stream)
}
```

### Request Batching and Queuing

```rust
// Batch API requests for efficiency
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub struct RequestBatcher<T> {
    sender: mpsc::UnboundedSender<T>,
    batch_size: usize,
    flush_interval: Duration,
}

impl<T> RequestBatcher<T> {
    pub fn new<F>(
        batch_size: usize,
        flush_interval: Duration,
        processor: F,
    ) -> Self 
    where
        F: Fn(Vec<T>) + Send + 'static,
        T: Send + 'static,
    {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let mut batch = Vec::with_capacity(batch_size);
        let mut timer = interval(flush_interval);
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    item = receiver.recv() => {
                        match item {
                            Some(item) => {
                                batch.push(item);
                                if batch.len() >= batch_size {
                                    processor(std::mem::take(&mut batch));
                                }
                            }
                            None => break,
                        }
                    }
                    _ = timer.tick() => {
                        if !batch.is_empty() {
                            processor(std::mem::take(&mut batch));
                        }
                    }
                }
            }
        });
        
        Self { sender, batch_size, flush_interval }
    }
    
    pub fn send(&self, item: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(item)
    }
}
```

## 🌐 Browser Performance

### Browser Optimization

```rust
// Optimized browser configuration
use chromiumoxide::{Browser, BrowserConfig, LaunchOptions};

pub async fn create_optimized_browser() -> Result<Browser> {
    let launch_options = LaunchOptions::default()
        .headless(true)
        .sandbox(false) // Better performance in containers
        .args(vec![
            "--no-sandbox",
            "--disable-dev-shm-usage",
            "--disable-gpu",
            "--disable-software-rasterizer",
            "--disable-background-timer-throttling",
            "--disable-backgrounding-occluded-windows",
            "--disable-renderer-backgrounding",
            "--disable-features=TranslateUI",
            "--disable-extensions",
            "--disable-default-apps",
            "--disable-sync",
            "--metrics-recording-only",
            "--no-first-run",
            "--safebrowsing-disable-auto-update",
            "--password-store=basic",
            "--use-mock-keychain",
            "--memory-pressure-off",
            "--max_old_space_size=4096",
        ]);
    
    let browser = Browser::launch(launch_options).await?;
    Ok(browser)
}

// Page optimization
impl BrowserService {
    pub async fn optimize_page(&self, page: &Page) -> Result<()> {
        // Disable images for faster loading
        page.execute(r#"
            const style = document.createElement('style');
            style.textContent = 'img { display: none !important; }';
            document.head.appendChild(style);
        "#).await?;
        
        // Disable animations
        page.execute(r#"
            const style = document.createElement('style');
            style.textContent = '*, *::before, *::after { 
                animation-duration: 0.01ms !important; 
                animation-delay: -0.01ms !important; 
                transition-duration: 0.01ms !important; 
            }';
            document.head.appendChild(style);
        "#).await?;
        
        // Clear browser cache periodically
        page.execute("window.localStorage.clear(); window.sessionStorage.clear();").await?;
        
        Ok(())
    }
}
```

### Element Locator Optimization

```rust
// Cached and optimized selectors
use once_cell::sync::Lazy;
use std::collections::HashMap;

static SELECTOR_CACHE: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("qr_code", "[data-testid='qr-code']");
    map.insert("chat_input", "[data-testid='compose-btn-send']");
    map.insert("phone_input", "input[type='tel']");
    map.insert("send_button", "[data-testid='compose-btn-send']");
    map
});

impl Locators {
    pub fn get_optimized_selector(element: &str) -> Option<&'static str> {
        SELECTOR_CACHE.get(element).copied()
    }
    
    // Use more efficient element location strategies
    pub async fn find_element_fast(
        &self,
        page: &Page,
        selector: &str,
    ) -> Result<chromiumoxide::Element> {
        // Try faster methods first
        if let Ok(element) = page.find_element(selector).await {
            return Ok(element);
        }
        
        // Fallback to XPath if CSS fails
        let xpath_selector = self.css_to_xpath(selector);
        page.find_element_by_xpath(&xpath_selector).await
    }
}
```

## 💾 Database and Storage

### Efficient Session Storage

```rust
// Optimized session storage
use serde::{Deserialize, Serialize};
use tokio::fs;
use bincode;

#[derive(Serialize, Deserialize)]
pub struct SessionData {
    pub user_agent: String,
    pub cookies: Vec<Cookie>,
    pub local_storage: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct OptimizedSessionStore {
    base_path: PathBuf,
}

impl OptimizedSessionStore {
    pub async fn save_session_compressed(
        &self,
        session_id: &str,
        data: &SessionData,
    ) -> Result<()> {
        let path = self.base_path.join(format!("{}.bin", session_id));
        
        // Use bincode for faster serialization
        let encoded = bincode::serialize(data)?;
        
        // Compress data
        let compressed = lz4::compress(&encoded)?;
        
        fs::write(path, compressed).await?;
        Ok(())
    }
    
    pub async fn load_session_compressed(
        &self,
        session_id: &str,
    ) -> Result<SessionData> {
        let path = self.base_path.join(format!("{}.bin", session_id));
        
        let compressed = fs::read(path).await?;
        let decompressed = lz4::decompress(&compressed)?;
        let data = bincode::deserialize(&decompressed)?;
        
        Ok(data)
    }
}
```

### Caching Layer

```rust
// Multi-level caching
use moka::future::Cache;
use std::time::Duration;

pub struct CacheManager {
    // L1: In-memory cache (fastest)
    memory_cache: Cache<String, Arc<Contact>>,
    
    // L2: File-based cache (medium)
    file_cache: FileCache,
    
    // L3: Database (slowest)
    database: Option<Database>,
}

impl CacheManager {
    pub fn new() -> Self {
        let memory_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(300))
            .time_to_idle(Duration::from_secs(60))
            .build();
        
        Self {
            memory_cache,
            file_cache: FileCache::new("cache"),
            database: None,
        }
    }
    
    pub async fn get_contact(&self, id: &str) -> Result<Option<Arc<Contact>>> {
        // L1: Check memory cache
        if let Some(contact) = self.memory_cache.get(id).await {
            return Ok(Some(contact));
        }
        
        // L2: Check file cache
        if let Some(contact) = self.file_cache.get(id).await? {
            let contact = Arc::new(contact);
            self.memory_cache.insert(id.to_string(), contact.clone()).await;
            return Ok(Some(contact));
        }
        
        // L3: Check database
        if let Some(db) = &self.database {
            if let Some(contact) = db.get_contact(id).await? {
                let contact = Arc::new(contact);
                self.file_cache.set(id, &contact).await?;
                self.memory_cache.insert(id.to_string(), contact.clone()).await;
                return Ok(Some(contact));
            }
        }
        
        Ok(None)
    }
}
```

## 🔄 Concurrency and Async

### Async Runtime Optimization

```rust
// Custom Tokio runtime configuration
use tokio::runtime::{Builder, Runtime};

pub fn create_optimized_runtime() -> Result<Runtime> {
    Builder::new_multi_thread()
        .worker_threads(num_cpus::get()) // Match CPU cores
        .max_blocking_threads(512)       // More blocking threads
        .thread_keep_alive(Duration::from_secs(60))
        .thread_stack_size(2 * 1024 * 1024) // 2MB stack
        .enable_all()
        .build()
}

// Task scheduling optimization
use tokio::task::LocalSet;

pub async fn process_messages_efficiently(messages: Vec<Message>) -> Result<()> {
    let local = LocalSet::new();
    
    local.run_until(async move {
        let mut tasks = Vec::new();
        
        for chunk in messages.chunks(10) {
            let chunk = chunk.to_vec();
            let task = tokio::task::spawn_local(async move {
                process_message_chunk(chunk).await
            });
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        futures::future::try_join_all(tasks).await?;
        Ok::<(), WhatsAppError>(())
    }).await
}
```

### Lock-free Data Structures

```rust
// Use lock-free alternatives when possible
use crossbeam::queue::SegQueue;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MessageQueue {
    queue: SegQueue<Message>,
    size: AtomicUsize,
}

impl MessageQueue {
    pub fn push(&self, message: Message) {
        self.queue.push(message);
        self.size.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn pop(&self) -> Option<Message> {
        match self.queue.pop() {
            Some(message) => {
                self.size.fetch_sub(1, Ordering::Relaxed);
                Some(message)
            }
            None => None,
        }
    }
    
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
}

// Arc instead of Mutex where possible
use arc_swap::ArcSwap;

pub struct ConfigManager {
    config: ArcSwap<AppConfig>,
}

impl ConfigManager {
    pub fn update_config(&self, new_config: AppConfig) {
        self.config.store(Arc::new(new_config));
    }
    
    pub fn get_config(&self) -> Arc<AppConfig> {
        self.config.load_full()
    }
}
```

## 🏪 Caching Strategies

### Application-level Caching

```rust
// Intelligent caching with TTL and size limits
use moka::future::Cache;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct CacheKey {
    operation: String,
    parameters: Vec<String>,
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.operation.hash(state);
        for param in &self.parameters {
            param.hash(state);
        }
    }
}

pub struct WhatsAppCache {
    // Different caches for different data types
    contacts: Cache<String, Contact>,
    chats: Cache<String, Chat>,
    messages: Cache<CacheKey, Vec<Message>>,
    auth_states: Cache<String, AuthStatus>,
}

impl WhatsAppCache {
    pub fn new() -> Self {
        Self {
            contacts: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(3600))
                .build(),
            
            chats: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(300))
                .build(),
            
            messages: Cache::builder()
                .max_capacity(100_000)
                .time_to_live(Duration::from_secs(60))
                .build(),
            
            auth_states: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(30))
                .build(),
        }
    }
    
    pub async fn get_or_compute<F, T>(&self, key: CacheKey, computer: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
        T: Clone + Send + Sync + 'static,
    {
        match self.get(&key).await {
            Some(value) => Ok(value),
            None => {
                let computed = computer()?;
                self.insert(key, computed.clone()).await;
                Ok(computed)
            }
        }
    }
}
```

### Redis Integration

```rust
// Redis for distributed caching
use redis::{Client, Commands, Connection};
use serde_json;

pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }
    
    pub async fn get_cached_result<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut conn = self.client.get_connection()?;
        let cached: Option<String> = conn.get(key)?;
        
        match cached {
            Some(data) => {
                let result = serde_json::from_str(&data)?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }
    
    pub async fn cache_result<T>(&self, key: &str, value: &T, ttl: usize) -> Result<()>
    where
        T: serde::Serialize,
    {
        let mut conn = self.client.get_connection()?;
        let serialized = serde_json::to_string(value)?;
        conn.set_ex(key, serialized, ttl)?;
        Ok(())
    }
}
```

## 🎛️ Resource Management

### Connection Pooling

```rust
// Browser instance pooling
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct BrowserPool {
    browsers: Vec<Arc<Browser>>,
    semaphore: Arc<Semaphore>,
    current: AtomicUsize,
}

impl BrowserPool {
    pub async fn new(pool_size: usize) -> Result<Self> {
        let mut browsers = Vec::with_capacity(pool_size);
        
        for _ in 0..pool_size {
            let browser = create_optimized_browser().await?;
            browsers.push(Arc::new(browser));
        }
        
        Ok(Self {
            browsers,
            semaphore: Arc::new(Semaphore::new(pool_size)),
            current: AtomicUsize::new(0),
        })
    }
    
    pub async fn acquire(&self) -> Result<BrowserGuard> {
        let _permit = self.semaphore.acquire().await?;
        let index = self.current.fetch_add(1, Ordering::Relaxed) % self.browsers.len();
        let browser = self.browsers[index].clone();
        
        Ok(BrowserGuard {
            browser,
            _permit,
        })
    }
}

pub struct BrowserGuard {
    browser: Arc<Browser>,
    _permit: tokio::sync::SemaphorePermit<'static>,
}

impl Deref for BrowserGuard {
    type Target = Browser;
    
    fn deref(&self) -> &Self::Target {
        &self.browser
    }
}
```

### Memory Management

```rust
// Smart memory management
use std::sync::atomic::{AtomicU64, Ordering};

pub struct MemoryManager {
    allocated_bytes: AtomicU64,
    max_memory: u64,
}

impl MemoryManager {
    pub fn new(max_memory_mb: u64) -> Self {
        Self {
            allocated_bytes: AtomicU64::new(0),
            max_memory: max_memory_mb * 1024 * 1024,
        }
    }
    
    pub fn check_memory_pressure(&self) -> bool {
        let current = self.allocated_bytes.load(Ordering::Relaxed);
        current > self.max_memory * 8 / 10 // 80% threshold
    }
    
    pub async fn cleanup_if_needed(&self) -> Result<()> {
        if self.check_memory_pressure() {
            // Trigger garbage collection
            self.force_cleanup().await?;
        }
        Ok(())
    }
    
    async fn force_cleanup(&self) -> Result<()> {
        // Clear caches
        GLOBAL_CACHE.clear().await;
        
        // Force GC in browser instances
        for browser in BROWSER_POOL.get_all().await {
            browser.call_function_on("() => { window.gc && window.gc(); }").await?;
        }
        
        Ok(())
    }
}
```

## 🏭 Production Optimization

### Deployment Configuration

```toml
# config/production.toml
[browser]
headless = true
timeout_ms = 30000
args = [
    "--no-sandbox",
    "--disable-dev-shm-usage",
    "--disable-gpu",
    "--memory-pressure-off",
    "--max_old_space_size=4096",
    "--optimize-for-size",
]

[performance]
max_concurrent_requests = 100
browser_pool_size = 5
cache_size_mb = 512
cleanup_interval_minutes = 30

[monitoring]
metrics_enabled = true
profiling_enabled = false  # Disable in production
```

### Kubernetes Optimization

```yaml
# k8s/deployment-optimized.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: whatsapp-engine-optimized
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: whatsapp-engine
        image: whatsapp-engine:optimized
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        
        env:
        - name: RUST_LOG
          value: "info"
        - name: BROWSER_POOL_SIZE
          value: "3"
        - name: MAX_CONCURRENT_REQUESTS
          value: "50"
        
        # Optimize for performance
        securityContext:
          capabilities:
            add:
            - SYS_ADMIN  # Required for Chrome sandboxing
        
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: dev-shm
          mountPath: /dev/shm
        
      volumes:
      - name: tmp
        emptyDir:
          sizeLimit: 1Gi
      - name: dev-shm
        emptyDir:
          medium: Memory
          sizeLimit: 1Gi
      
      # Node affinity for performance
      affinity:
        nodeAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            nodeSelectorTerms:
            - matchExpressions:
              - key: performance
                operator: In
                values: ["high"]
```

### Load Balancing

```yaml
# nginx.conf for load balancing
upstream whatsapp_backend {
    least_conn;
    server whatsapp-1:3000 max_fails=3 fail_timeout=30s;
    server whatsapp-2:3000 max_fails=3 fail_timeout=30s;
    server whatsapp-3:3000 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    
    # Connection optimization
    keepalive_timeout 65;
    keepalive_requests 100;
    
    # Compression
    gzip on;
    gzip_types text/plain application/json;
    
    location / {
        proxy_pass http://whatsapp_backend;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## 📊 Monitoring Performance

### Performance Metrics

```rust
// Custom performance monitoring
use prometheus::{Histogram, Counter, Gauge};

lazy_static! {
    static ref REQUEST_DURATION: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new("request_duration_seconds", "Request duration")
            .buckets(vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0])
    ).unwrap();
    
    static ref MEMORY_USAGE: Gauge = Gauge::with_opts(
        prometheus::GaugeOpts::new("memory_usage_bytes", "Memory usage in bytes")
    ).unwrap();
    
    static ref BROWSER_OPERATIONS: Counter = Counter::with_opts(
        prometheus::CounterOpts::new("browser_operations_total", "Browser operations")
    ).unwrap();
}

pub fn record_performance_metrics() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            // Update memory usage
            if let Ok(usage) = get_memory_usage() {
                MEMORY_USAGE.set(usage as f64);
            }
            
            // Update other system metrics
            update_system_metrics().await;
        }
    });
}
```

### Performance Alerts

```yaml
# performance-alerts.yml
groups:
- name: performance
  rules:
  - alert: HighLatency
    expr: |
      histogram_quantile(0.95, 
        rate(request_duration_seconds_bucket[5m])
      ) > 5
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "High request latency detected"
  
  - alert: HighMemoryUsage
    expr: memory_usage_bytes > 1073741824  # 1GB
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Memory usage too high"
```

This comprehensive performance guide provides strategies and implementations for optimizing WhatsApp Engine Rust across all performance dimensions.
