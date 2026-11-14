#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Repeat {
    #[default]
    Off,
    All,
    One,
}
impl Repeat {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::All,
            Self::All => Self::One,
            Self::One => Self::Off,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Repeat off",
            Self::All => "Repeat all",
            Self::One => "Repeat one",
        }
    }
    pub fn number(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::All => 1,
            Self::One => 2,
        }
    }
    pub fn from_number(n: i32) -> Self {
        match n {
            1 => Self::All,
            2 => Self::One,
            _ => Self::Off,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Queue {
    pub canonical: Vec<usize>,
    pub order: Vec<usize>,
    pub cursor: usize,
    pub shuffle: bool,
    pub repeat: Repeat,
    pub position: f64,
}
impl Queue {
    pub fn current(&self) -> Option<usize> {
        self.order.get(self.cursor).copied()
    }
    pub fn replace(&mut self, ids: Vec<usize>, selected: usize) {
        let mut seen = std::collections::BTreeSet::new();
        self.canonical = ids.into_iter().filter(|id| seen.insert(*id)).collect();
        if !seen.contains(&selected) {
            self.canonical.push(selected);
        }
        self.order = self.canonical.clone();
        self.cursor = self
            .order
            .iter()
            .position(|id| *id == selected)
            .unwrap_or(0);
        self.position = 0.0;
        if self.shuffle {
            self.shuffle_remaining();
        }
    }
    pub fn can_next(&self) -> bool {
        self.current().is_some()
            && (self.cursor + 1 < self.order.len() || self.repeat == Repeat::All)
    }
    pub fn can_previous(&self) -> bool {
        self.current().is_some() && (self.cursor > 0 || self.repeat == Repeat::All)
    }
    pub fn advance(&mut self, automatic: bool) -> Option<usize> {
        if automatic && self.repeat == Repeat::One {
            self.position = 0.0;
            return self.current();
        }
        if self.cursor + 1 < self.order.len() {
            self.cursor += 1;
        } else if self.repeat == Repeat::All && !self.order.is_empty() {
            let previous = self.current();
            self.cursor = 0;
            if self.shuffle {
                Self::shuffle_slice(&mut self.order);
                if self.order.len() > 1 && self.current() == previous {
                    self.order.swap(0, 1);
                }
            }
        } else {
            return None;
        }
        self.position = 0.0;
        self.current()
    }
    pub fn previous(&mut self) -> Option<usize> {
        if self.cursor > 0 {
            self.cursor -= 1;
        } else if self.repeat == Repeat::All && !self.order.is_empty() {
            self.cursor = self.order.len() - 1;
        } else {
            return None;
        }
        self.position = 0.0;
        self.current()
    }
    pub fn set_shuffle(&mut self, enabled: bool) {
        if self.shuffle == enabled {
            return;
        }
        let current = self.current();
        self.shuffle = enabled;
        if enabled {
            self.shuffle_remaining();
        } else {
            self.order = self.canonical.clone();
            self.cursor = current
                .and_then(|id| self.order.iter().position(|n| *n == id))
                .unwrap_or(0);
        }
    }
    fn shuffle_remaining(&mut self) {
        let first = (self.cursor + 1).min(self.order.len());
        Self::shuffle_slice(&mut self.order[first..]);
    }
    fn shuffle_slice(order: &mut [usize]) {
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
            | 1;
        for i in (1..order.len()).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let j = seed as usize % (i + 1);
            order.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eof_stops_but_manual_next_ignores_repeat_one() {
        let mut q = Queue::default();
        q.replace(vec![2, 8], 2);
        q.repeat = Repeat::One;
        assert_eq!(q.advance(true), Some(2));
        assert_eq!(q.advance(false), Some(8));
        q.repeat = Repeat::Off;
        assert_eq!(q.advance(true), None);
        assert_eq!(q.current(), Some(8));
    }
    #[test]
    fn repeat_all_wraps_both_directions() {
        let mut q = Queue::default();
        q.replace(vec![4, 7], 7);
        q.repeat = Repeat::All;
        assert_eq!(q.advance(true), Some(4));
        assert_eq!(q.previous(), Some(7));
    }
    #[test]
    fn shuffle_preserves_current_history_identity_and_original_order() {
        let mut q = Queue::default();
        q.replace(vec![1, 2, 3, 4, 5, 6], 3);
        q.set_shuffle(true);
        assert_eq!(&q.order[..=2], &[1, 2, 3]);
        let mut sorted = q.order.clone();
        sorted.sort();
        assert_eq!(sorted, q.canonical);
        let id = q.advance(false).unwrap();
        q.set_shuffle(false);
        assert_eq!(q.current(), Some(id));
        assert_eq!(q.order, q.canonical);
    }
    #[test]
    fn replacement_deduplicates_and_does_not_lose_selected_track() {
        let mut q = Queue::default();
        q.replace(vec![5, 5, 3], 9);
        assert_eq!(q.order, vec![5, 3, 9]);
        assert_eq!(q.current(), Some(9));
    }
    #[test]
    fn shuffled_repeat_cycle_keeps_membership_without_immediate_repeat() {
        let mut q = Queue::default();
        q.replace(vec![1, 2, 3, 4], 4);
        q.set_shuffle(true);
        q.repeat = Repeat::All;
        assert_ne!(q.advance(true), Some(4));
        let mut ids = q.order.clone();
        ids.sort();
        assert_eq!(ids, vec![1, 2, 3, 4]);
    }
}
