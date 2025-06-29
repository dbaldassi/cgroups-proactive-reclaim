
use std::{fs, cmp};
use std::io::{self, BufRead};

use csv::Writer;
use chrono::Local;

const MEMORY_MAX: &str = "memory.max";
const MEMORY_CURRENT: &str = "memory.current";
const MEMORY_STAT: &str = "memory.stat";
const MEMORY_RECLAIM: &str = "memory.reclaim";
const SWAP_MAX: &str = "memory.swap.max";
const SWAP_CURRENT: &str = "memory.swap.current";
const WINDOW_SIZE: usize = 30; // Size of the sliding window for standard deviation calculation
const STDDEV_THRESHOLD: f64 = 1.0; // Threshold for standard deviation to trigger proactive reclaim
const CGROUPS_MAX_RECLAIM: u64 = 100 * 1024 * 1024; // Maximum reclaim value for cgroups

#[derive(serde::Serialize, Default)]
struct MemoryStat {
    inactive_anon: u64,
    active_anon: u64,
    inactive_file: u64,
    active_file: u64,
    current_memory_usage: u64, // Current memory usage
    current_swap_usage: u64, // Current swap usage
    memory_max: u64, // Maximum memory limit
    swap_max: u64, // Maximum swap limit
}

#[derive(serde::Serialize, Default)]
struct LogEntry {
    timestamp: u64,
    current_memory_usage: u64, // Current memory usage
    current_swap_usage: u64, // Current swap usage
    memory_max: u64, // Maximum memory limit
    active_anon: u64,
    inactive_anon: u64,
    swap_max: u64, // Maximum swap limit
    active_file: u64,
    inactive_file: u64,
}

pub struct CgroupsReclaimManager {
    domain: String, // Path to the cgroup

    memory_stat: MemoryStat, // Memory statistics for standard deviation calculation

    memory_max_path: String, // Path to memory.max
    memory_current_path: String, // Path to memory.current
    memory_stat_path: String, // Path to memory.stat
    memory_reclaim_path: String, // Path to memory.reclaim
    swap_max_path: String, // Path to memory.swap.max
    swap_current_path: String, // Path to memory.swap.current

    logger: Option<csv::Writer<std::fs::File>>, // Optional CSV logger for memory statistics

    window: Vec<f64>, // Sliding window for standard deviation calculation
}

impl CgroupsReclaimManager {
    pub fn new(domain: &str) -> Self {
        let cgroup_path = get_cgroup_path(domain).unwrap_or_else(|_| {
            panic!("Failed to get cgroup path for domain: {}", domain);
        });

        CgroupsReclaimManager {
            domain: domain.to_string(),
            memory_stat: MemoryStat {
                ..MemoryStat::default() // Initialize memory statistics
            },
            memory_max_path: format!("{}/{}", cgroup_path, MEMORY_MAX),
            memory_current_path: format!("{}/{}", cgroup_path, MEMORY_CURRENT),
            memory_stat_path: format!("{}/{}", cgroup_path, MEMORY_STAT),
            memory_reclaim_path: format!("{}/{}", cgroup_path, MEMORY_RECLAIM),
            swap_max_path: format!("{}/{}", cgroup_path, SWAP_MAX),
            swap_current_path: format!("{}/{}", cgroup_path, SWAP_CURRENT),
            window: Vec::with_capacity(WINDOW_SIZE), // Initialize sliding window
            logger: None, // Initialize logger as None
        }
    }

    fn stddev(&self, values: &[f64]) -> f64 {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }

    fn update_window(&mut self) {
        if self.window.len() >= WINDOW_SIZE {
            self.window.remove(0); // Remove the oldest value if the window is full
        } 
        
        self.window.push(self.memory_stat.inactive_anon as f64);
    }

    fn get_statistics(&mut self) -> Result<(), String> {
        // read memory statistics from the cgroup
        
        let file = fs::File::open(&self.memory_stat_path)
            .map_err(|e| format!("Failed to open memory.stat: {}", e))?;
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                match parts[0] {
                    "inactive_anon" => self.memory_stat.inactive_anon = parts[1].trim().parse().unwrap_or(0),
                    "active_anon" => self.memory_stat.active_anon = parts[1].trim().parse().unwrap_or(0),
                    "inactive_file" => self.memory_stat.inactive_file = parts[1].trim().parse().unwrap_or(0),
                    "active_file" => self.memory_stat.active_file = parts[1].trim().parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

        // Read memory current usage
        let contents = fs::read_to_string(self.memory_current_path.clone())
            .expect("Should have been able to read the file");
        self.memory_stat.current_memory_usage = contents.trim().parse::<u64>().unwrap_or(0);
        // Read swap current usage
        let contents = fs::read_to_string(self.swap_current_path.clone())
            .expect("Should have been able to read the file");
        self.memory_stat.current_swap_usage = contents.trim().parse::<u64>().unwrap_or(0);
        // Read memory max
        let contents = fs::read_to_string(self.memory_max_path.clone())
            .expect("Should have been able to read the file");
        self.memory_stat.memory_max = contents.trim().parse::<u64>().unwrap_or(0);
        // Read swap max
        let contents = fs::read_to_string(self.swap_max_path.clone())
            .expect("Should have been able to read the file");
        self.memory_stat.swap_max = contents.trim().parse::<u64>().unwrap_or(0);


        Ok(())
    }   

    pub fn set_max_memory(&self, max_memory: u64) -> Result<(), String> {
        fs::write(&self.memory_max_path, max_memory.to_string())
            .map_err(|e| format!("Failed to set memory.max: {}", e))?;
        Ok(())
    }

    pub fn reclaim_memory(&self, value: u64) -> Result<(), String> {
        // Placeholder for memory reclaim logic
        // This would involve writing to the cgroup's memory.reclaim file
        fs::write(&self.memory_reclaim_path, value.to_string())
            .map_err(|e| format!("Failed to reclaim memory: {}", e))?;
        Ok(())
    }

    fn get_initial_memory_reclaim(&self) -> u64 {
        1024 * 1024 * 15 // 15 MB
    }

    pub fn regulate(&mut self, free_memory: u64, safety: u64) -> Result<(), String> {
        // Placeholder for proactive reclaim logic
        // This would involve checking the cgroup's resource usage and reclaiming if necessary

        self.get_statistics()?;

        let unused = self.memory_stat.current_memory_usage - free_memory;

        self.update_window();

        if unused < safety {
            // Error::new(io::ErrorKind::Other, "Free memory below safety")
            println!("Unused memory below safety, not reclaiming memory");
        } else {
            // Perform reclaim logic
            let inactive_anon = self.memory_stat.inactive_anon; // Convert to MB

            // create inactive list
            if inactive_anon == 0 {
                self.reclaim_memory(self.get_initial_memory_reclaim())?;
                return Ok(());
            }

            if self.window.len() < WINDOW_SIZE {
                println!("Not enough data in the sliding window to calculate standard deviation");
                return Ok(());
            }

            println!("Check stabilization");
            let stddev = self.stddev(&self.window);

            if stddev < STDDEV_THRESHOLD {
                println!("Standard deviation is above threshold, reclaiming memory");
                self.reclaim_memory(cmp::min(CGROUPS_MAX_RECLAIM, unused - safety))?;
                self.window.clear(); // Clear the window after reclaim
            }
        }
        
        Ok(())
    }

    fn create_csv_writer(&mut self) {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

        let path =  format!("cgroup_{}_{}.csv", &self.domain, timestamp);
        let file = std::fs::File::create(path).expect("Failed to create CSV file");
        self.logger = Some(Writer::from_writer(file));
    }

    pub fn dump_mem_stats(&mut self, timestamp: u64) {
        if self.logger.is_none() {
            self.create_csv_writer();
        }

        if let Some(ref mut logger) = self.logger {
            let log_entry = LogEntry {
                timestamp,
                current_memory_usage: self.memory_stat.current_memory_usage,
                current_swap_usage: self.memory_stat.current_swap_usage,
                memory_max: self.memory_stat.memory_max,
                active_anon: self.memory_stat.active_anon,  
                inactive_anon: self.memory_stat.inactive_anon,
                swap_max: self.memory_stat.swap_max,
                active_file: self.memory_stat.active_file,
                inactive_file: self.memory_stat.inactive_file,
            };

            logger.serialize(log_entry).expect("Failed to write to CSV");
        } else {
            println!("Logger not initialized, cannot dump memory stats");
        }    
    }
    
    // Add methods to manage cgroups and reclaim resources
}

pub fn get_cgroup_path(domain_name: &str) -> Result<String,()> {
    const CGROUP_BASE_PATH: &str = "/sys/fs/cgroup/machine.slice";
    let pattern = format!("{}.scope", domain_name);

    if let Ok(entries) = fs::read_dir(CGROUP_BASE_PATH) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() && path.to_string_lossy().contains(&pattern) {
                    return Ok(path.to_string_lossy().to_string());
                }
            }
        }
    }

    // If no matching cgroup is found, return an empty string or handle the error as needed
    println!("No matching cgroup found for domain: {}", domain_name);
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
