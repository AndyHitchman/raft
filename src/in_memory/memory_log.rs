//! Memory Log is actually volatile and is used to support
//! development, testing and experiments.
//!
//! Don't use MemoryLog with a real workload.
use server::log::*;

pub struct MemoryLog {
    log:    Vec<LogEntry>,
}

impl MemoryLog {
    pub fn new() -> MemoryLog {
        MemoryLog {log: vec![]}
    }
}

impl LogPersister for MemoryLog {
    fn append_entry(&mut self, entry: LogEntry) -> Result<(), LogFailed> {
        self.log.push(entry);
        Result::Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use server::log::*;

    #[test]
    fn append_entry_should_append_to_log() {
        let mut ml = MemoryLog::new();
        let term = 1;
        let entry = vec![1u8, 2u8];
        let log_entry = LogEntry::new(1, &entry);

        ml.append_entry(log_entry);

        assert!(ml.log.len() == 1);
        assert_eq!(ml.log[0].term, term);
        assert_eq!(ml.log[0].entry, entry)
    }
}
