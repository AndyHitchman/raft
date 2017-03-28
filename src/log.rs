pub struct LogEntry {
    pub term:   u64,
    pub entry:  Vec<u8>,
}

// impl PartialEq for LogEntry {
//     fn eq(&self, other: &LogEntry) -> bool {
//         self.term == other.term && self.entry == other.entry
//     }
// }

impl LogEntry {
    pub fn new(term: u64, entry: &Vec<u8>) -> LogEntry {
        LogEntry {
            term:   term,
            entry:  entry.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_should_return_a_log_entry_with_the_suppled_values() {
        let term = 2;
        let entry = vec![1u8, 2u8, 3u8, 4u8, 5u8];

        let actual = LogEntry::new(term, &entry);

        assert_eq!(actual.term, term);
        assert_eq!(actual.entry, entry);
    }
}

pub enum LogFailed {
    NoFreeSpace,
    InsufficientPermission,
    OSError(String)
}

pub trait LogPersister : Send + Sync {
    fn append_entry(&mut self, entry: LogEntry) -> Result<(), LogFailed>;
}
