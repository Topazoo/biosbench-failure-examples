//! Generic adaptation of the project session guard; identifiers are opaque data.
//! Equality of caller-supplied tokens is not authentication or exclusive ownership.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub epoch: u64,
    pub owner: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    Identity,
    Operation(E),
}

pub struct Session<T> {
    value: T,
    identity: Option<Identity>,
    sealed: bool,
}

impl<T> Session<T> {
    pub fn new(value: T) -> Self {
        Self { value, identity: None, sealed: false }
    }

    pub fn bind(&mut self, identity: Identity) -> Result<(), Error<()>> {
        if self.sealed || self.identity.is_some() || identity.epoch == 0 || identity.owner == 0 {
            self.close();
            return Err(Error::Identity);
        }
        self.identity = Some(identity);
        Ok(())
    }

    pub fn close(&mut self) {
        self.identity = None;
        self.sealed = true;
    }

    pub fn run<R, E>(&mut self, current: Identity,
                     operation: impl FnOnce(&mut T) -> Result<R, E>) -> Result<R, Error<E>> {
        if self.sealed || self.identity != Some(current) {
            self.close();
            return Err(Error::Identity);
        }
        // Seal before fallible user code so even unwinding cannot reopen access.
        self.sealed = true;
        match operation(&mut self.value) {
            Ok(result) => { self.sealed = false; Ok(result) }
            Err(error) => { self.close(); Err(Error::Operation(error)) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: Identity = Identity { epoch: 1, owner: 2 };

    #[test]
    fn same_session_and_terminal_close() {
        let mut session = Session::new(0);
        session.bind(ID).unwrap();
        assert_eq!(session.run(ID, |v| { *v += 1; Ok::<_, ()>(*v) }), Ok(1));
        session.close();
        assert_eq!(session.run(ID, |_| Ok::<_, ()>(2)), Err(Error::Identity));
        assert!(session.bind(ID).is_err());
    }

    #[test]
    fn identity_drift_never_invokes_operation_and_is_terminal() {
        for changed in [Identity { epoch: 2, ..ID }, Identity { owner: 3, ..ID }] {
            let mut session = Session::new(());
            session.bind(ID).unwrap();
            assert_eq!(session.run(changed, |_| -> Result<(), ()> { panic!("invoked") }),
                       Err(Error::Identity));
            assert!(session.run(ID, |_| Ok::<_, ()>(())).is_err());
        }
    }

    #[test]
    fn unbound_invalid_and_rebind_are_terminal() {
        let mut unbound = Session::new(());
        assert!(unbound.run(ID, |_| Ok::<_, ()>(())).is_err());
        assert!(unbound.bind(ID).is_err());
        for identity in [Identity { epoch: 0, ..ID }, Identity { owner: 0, ..ID }] {
            let mut session = Session::new(());
            assert!(session.bind(identity).is_err());
            assert!(session.bind(ID).is_err());
        }
        let mut session = Session::new(());
        session.bind(ID).unwrap();
        assert!(session.bind(ID).is_err());
        assert!(session.run(ID, |_| Ok::<_, ()>(())).is_err());
    }

    #[test]
    fn failure_and_panic_are_terminal() {
        let mut session = Session::new(());
        session.bind(ID).unwrap();
        assert_eq!(session.run(ID, |_| Err::<(), _>("injected")), Err(Error::Operation("injected")));
        assert!(session.run(ID, |_| Ok::<_, ()>(())).is_err());
        let mut panicking = Session::new(());
        panicking.bind(ID).unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = panicking.run(ID, |_| -> Result<(), ()> { panic!("injected") });
        }));
        assert!(result.is_err());
        assert!(panicking.run(ID, |_| Ok::<_, ()>(())).is_err());
    }
}
