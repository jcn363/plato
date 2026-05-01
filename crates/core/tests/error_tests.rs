#[cfg(test)]
mod tests {
    use crate::into_plato_err;
    use thiserror::Error;

    #[derive(Debug, Error)]
    enum Dummy {
        #[error("dummy error")]
        Dummy,
    }

    #[test]
    fn test_into_plato_err() {
        let e = Dummy::Dummy;
        let err = into_plato_err(e);
        // The error should contain the original message.
        if let crate::PlatoError::Ai(inner) = err {
            assert!(inner.to_string().contains("dummy error"));
        } else {
            panic!("Expected PlatoError::Ai");
        }
    }
}
