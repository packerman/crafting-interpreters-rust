use std::ops::Index;

#[derive(Debug, Default)]
pub struct RunLength<T> {
    entries: Vec<Entry<T>>,
}

impl<T: PartialEq> RunLength<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, element: T) {
        match self.entries.last_mut() {
            Some(entry) if entry.element == element => {
                entry.count += 1;
            }
            _ => self.entries.push(Entry::new(element, 1)),
        }
    }
}

#[derive(Debug)]
struct Entry<T> {
    element: T,
    count: usize,
}

impl<T> Entry<T> {
    fn new(element: T, count: usize) -> Self {
        Self { element, count }
    }
}

impl<T> Index<usize> for RunLength<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let mut offset = index;
        for entry in self.entries.iter() {
            if offset < entry.count {
                return &entry.element;
            }
            offset -= entry.count;
        }
        panic!("Index out of bounds.");
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn identity_exprs_keys() {
        let mut v = RunLength::new();

        v.push('a');
        v.push('b');
        v.push('b');
        v.push('b');
        v.push('c');
        v.push('c');

        assert_eq!(v[0], 'a');
        assert_eq!(v[1], 'b');
        assert_eq!(v[2], 'b');
        assert_eq!(v[3], 'b');
        assert_eq!(v[4], 'c');
        assert_eq!(v[5], 'c');
    }
}
