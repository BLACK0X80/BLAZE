use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Profiler {
    metrics: HashMap<String, MetricData>,
    current_scope: Vec<String>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct MetricData {
    pub total_time: Duration,
    pub call_count: u64,
    pub min_time: Duration,
    pub max_time: Duration,
    pub self_time: Duration,
}

pub struct ProfileScope<'a> {
    profiler: &'a mut Profiler,
    name: String,
    start: Instant,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            current_scope: Vec::new(),
            enabled: true,
        }
    }
    
    pub fn start_scope<'a>(&'a mut self, name: String) -> ProfileScope<'a> {
        if self.enabled {
            self.current_scope.push(name.clone());
        }
        
        ProfileScope {
            profiler: self,
            name,
            start: Instant::now(),
        }
    }
    
    fn end_scope(&mut self, name: &str, duration: Duration) {
        if !self.enabled {
            return;
        }
        
        let metric = self.metrics.entry(name.to_string()).or_insert(MetricData {
            total_time: Duration::ZERO,
            call_count: 0,
            min_time: Duration::from_secs(u64::MAX),
            max_time: Duration::ZERO,
            self_time: Duration::ZERO,
        });
        
        metric.total_time += duration;
        metric.call_count += 1;
        metric.min_time = metric.min_time.min(duration);
        metric.max_time = metric.max_time.max(duration);
        metric.self_time += duration;
        
        if let Some(parent) = self.current_scope.last() {
            if let Some(parent_metric) = self.metrics.get_mut(parent) {
                parent_metric.self_time = parent_metric.self_time.saturating_sub(duration);
            }
        }
        
        self.current_scope.pop();
    }
    
    pub fn get_metric(&self, name: &str) -> Option<&MetricData> {
        self.metrics.get(name)
    }
    
    pub fn get_all_metrics(&self) -> &HashMap<String, MetricData> {
        &self.metrics
    }
    
    pub fn clear(&mut self) {
        self.metrics.clear();
        self.current_scope.clear();
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn print_report(&self) {
        println!("\n=== Profiling Report ===\n");
        
        let mut sorted_metrics: Vec<_> = self.metrics.iter().collect();
        sorted_metrics.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));
        
        println!("{:<30} {:>12} {:>10} {:>12} {:>12} {:>12}", 
                 "Function", "Calls", "Total(ms)", "Avg(ms)", "Min(ms)", "Max(ms)");
        println!("{:-<90}", "");
        
        for (name, metric) in sorted_metrics {
            let avg = metric.total_time.as_millis() / metric.call_count as u128;
            
            println!("{:<30} {:>12} {:>10} {:>12} {:>12} {:>12}",
                     name,
                     metric.call_count,
                     metric.total_time.as_millis(),
                     avg,
                     metric.min_time.as_millis(),
                     metric.max_time.as_millis());
        }
        
        println!("\n");
    }
    
    pub fn export_json(&self) -> String {
        let mut json = String::from("{\n");
        
        for (i, (name, metric)) in self.metrics.iter().enumerate() {
            if i > 0 {
                json.push_str(",\n");
            }
            
            json.push_str(&format!(
                "  \"{}\": {{\n    \"calls\": {},\n    \"total_ms\": {},\n    \"avg_ms\": {},\n    \"min_ms\": {},\n    \"max_ms\": {}\n  }}",
                name,
                metric.call_count,
                metric.total_time.as_millis(),
                metric.total_time.as_millis() / metric.call_count as u128,
                metric.min_time.as_millis(),
                metric.max_time.as_millis()
            ));
        }
        
        json.push_str("\n}");
        json
    }
}

impl<'a> Drop for ProfileScope<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.profiler.end_scope(&self.name, duration);
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricData {
    pub fn average_time(&self) -> Duration {
        if self.call_count == 0 {
            Duration::ZERO
        } else {
            self.total_time / self.call_count as u32
        }
    }
}

#[macro_export]
macro_rules! profile {
    ($profiler:expr, $name:expr, $block:block) => {{
        let _scope = $profiler.start_scope($name.to_string());
        $block
    }};
}
