//! Going back and forward, like a web browser.

/// Where you are, and where you have been.
#[derive(Debug, Clone)]
pub struct History<Visit> {
    current: Option<Visit>,
    /// The visits behind the current one, the most recent last.
    back: Vec<Visit>,
    /// The visits left by going back, the most recent last.
    forward: Vec<Visit>,
}

impl<Visit> Default for History<Visit> {
    fn default() -> Self {
        Self {
            current: None,
            back: Vec::new(),
            forward: Vec::new(),
        }
    }
}

impl<Visit> History<Visit> {
    /// Where you are, if anywhere.
    #[must_use]
    pub fn current(&self) -> Option<&Visit> {
        self.current.as_ref()
    }

    /// Where you are, to change it.
    pub fn current_mut(&mut self) -> Option<&mut Visit> {
        self.current.as_mut()
    }

    /// Where going back would go.
    #[must_use]
    pub fn behind(&self) -> Option<&Visit> {
        self.back.last()
    }

    /// Where going forward would go.
    #[must_use]
    pub fn ahead(&self) -> Option<&Visit> {
        self.forward.last()
    }

    /// Every visit, the current one first, to change them.
    pub fn visits_mut(&mut self) -> impl Iterator<Item = &mut Visit> {
        let current = self.current.iter_mut();
        current
            .chain(self.back.iter_mut())
            .chain(self.forward.iter_mut())
    }

    /// Goes somewhere new, which clears the way forward.
    pub fn visit(&mut self, visit: Visit) {
        self.back.extend(self.current.replace(visit));
        self.forward.clear();
    }

    /// Goes back, if there is anything to go back to.
    pub fn back(&mut self) {
        if let Some(visit) = self.back.pop() {
            self.forward.extend(self.current.replace(visit));
        }
    }

    /// Goes forward, if there is anything to go forward to.
    pub fn forward(&mut self) {
        if let Some(visit) = self.forward.pop() {
            self.back.extend(self.current.replace(visit));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visited(places: &[&'static str]) -> History<&'static str> {
        let mut history = History::default();
        for place in places {
            history.visit(*place);
        }
        history
    }

    #[test]
    fn it_starts_nowhere_and_stays_there() {
        let mut history = visited(&[]);
        history.back();
        history.forward();
        assert_eq!(history.current(), None);
    }

    #[test]
    fn going_back_and_forward_returns_to_the_same_place() {
        let mut history = visited(&["a", "b", "c"]);
        history.back();
        history.back();
        assert_eq!(history.current(), Some(&"a"));
        history.forward();
        history.forward();
        assert_eq!(history.current(), Some(&"c"));
    }

    #[test]
    fn a_new_visit_clears_the_way_forward_but_not_the_way_back() {
        let mut history = visited(&["a", "b"]);
        history.back();
        history.visit("c");
        history.forward();
        assert_eq!(history.current(), Some(&"c"));
        history.back();
        assert_eq!(history.current(), Some(&"a"));
    }

    #[test]
    fn the_way_back_and_forward_can_be_seen_without_going() {
        let mut history = visited(&["a", "b", "c"]);
        history.back();
        assert_eq!(history.behind(), Some(&"a"));
        assert_eq!(history.ahead(), Some(&"c"));
        assert_eq!(history.current(), Some(&"b"));
    }

    #[test]
    fn every_visit_can_be_changed() {
        let mut history = visited(&["a", "b", "c"]);
        history.back();
        history.visits_mut().for_each(|visit| *visit = "z");
        history.back();
        assert_eq!(history.current(), Some(&"z"));
        history.forward();
        history.forward();
        assert_eq!(history.current(), Some(&"z"));
    }

    #[test]
    fn a_visit_keeps_what_was_changed_in_it() {
        let mut history = History::default();
        history.visit(1);
        if let Some(visit) = history.current_mut() {
            *visit = 2;
        }
        history.visit(3);
        history.back();
        assert_eq!(history.current(), Some(&2));
    }
}
