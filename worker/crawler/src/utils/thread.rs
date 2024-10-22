use num_cpus;

pub fn determine_thread_num() -> usize {
    let cpu_cores = num_cpus::get();
    std::cmp::max(1, cpu_cores - 1)
}
