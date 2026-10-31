//! Request epochs prevent an obsolete navigation response from restoring a former session.
#[derive(Default, Clone, Copy)]
pub struct SessionEpoch(u64);
impl SessionEpoch {
    pub fn invalidate(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
    pub fn begin(&mut self) -> u64 {
        self.invalidate();
        self.0
    }
    pub fn accepts(self, ticket: u64) -> bool {
        self.0 == ticket
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn logout_and_new_login_reject_both_older_responses() {
        let mut session = super::SessionEpoch::default();
        let old = session.begin();
        session.invalidate(); // logout in another tab, even before response arrives
        assert!(!session.accepts(old));
        let new = session.begin();
        assert!(!session.accepts(old));
        assert!(session.accepts(new));
        session.invalidate(); // another identity change
        assert!(!session.accepts(new));
    }
}
