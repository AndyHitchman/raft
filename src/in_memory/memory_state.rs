/// Memory State is actually volatile and is used to support
/// development, testing and experiments.
///
/// Don't use MemoryState with a real workload.
use server::state::*;

pub struct MemoryState;

impl MemoryState {
    pub fn new() -> MemoryState {
        MemoryState
    }
}

impl StatePersister for MemoryState {
    /// Does nothing
    fn endure(&self, state: &PersistentState) -> Result<(), PersistFailed> {
        Ok(())
    }

    /// Pretends the server is starting for the first time.
    fn load(&self) -> Result<PersistentState, PersistFailed> {
        Ok(PersistentState::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use server::state::*;

    #[test]
    fn endure_should_return_ok() {
        let mut ms = MemoryState::new();

        assert!(ms.endure(&PersistentState::new()).is_ok())
    }

    #[test]
    fn load_should_return_new_persistent_state() {
        let mut ms = MemoryState::new();

        let actual = ms.load().ok().unwrap();

        assert_eq!(actual.current_term, 0);
        assert!(actual.voted_for.is_none());
    }
}
