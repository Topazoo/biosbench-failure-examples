//! Consume the fixture's submit permission before any fallible routing work.
//! This does not establish session ownership or authorize motherboard writes.
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> { Denied, Operation(E) }

pub fn run<T, E>(permit: &AtomicUsize, operation: impl FnOnce() -> Result<T, E>) -> Result<T, Error<E>> {
    if permit.swap(0, Ordering::SeqCst) != 1 { return Err(Error::Denied); }
    operation().map_err(Error::Operation)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_failure_stage_consumes_permission_and_prevents_retry() {
        // Decode, baseline retrieval, dependency check, write, readback.
        for failed_stage in 0..5 {
            let permit = AtomicUsize::new(1);
            let mut reached = 0;
            assert_eq!(run(&permit, || {
                for stage in 0..5 {
                    reached += 1;
                    if stage == failed_stage { return Err(stage); }
                }
                Ok(())
            }), Err(Error::Operation(failed_stage)));
            assert_eq!(reached, failed_stage + 1);
            assert_eq!(run(&permit, || -> Result<(), usize> { panic!("retry executed") }), Err(Error::Denied));
        }
    }
    #[test]
    fn successful_operation_is_also_one_use() {
        let permit = AtomicUsize::new(1);
        assert_eq!(run(&permit, || Ok::<_, ()>(42)), Ok(42));
        assert_eq!(run(&permit, || -> Result<(), ()> { panic!("second operation") }), Err(Error::Denied));
    }
    #[test]
    fn reentrant_operation_is_denied_before_it_runs() {
        let permit = AtomicUsize::new(1);
        assert_eq!(run(&permit, || {
            assert_eq!(run(&permit, || -> Result<(), ()> { panic!("reentry") }), Err(Error::Denied));
            Ok::<_, ()>(())
        }), Ok(()));
    }
    #[test]
    fn absent_or_invalid_permission_never_invokes_operation() {
        for value in [0, 2, usize::MAX] {
            let permit = AtomicUsize::new(value);
            assert_eq!(run(&permit, || -> Result<(), ()> { panic!("unauthorized operation") }), Err(Error::Denied));
            assert_eq!(permit.load(Ordering::SeqCst), 0);
        }
    }
}
