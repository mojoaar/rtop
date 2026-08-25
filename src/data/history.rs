#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buf: Vec<T>,
    head: usize,
    cap: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(cap: usize) -> Self {
        Self { buf: Vec::with_capacity(cap), head: 0, cap: cap.max(1) }
    }

    pub fn push(&mut self, value: T) {
        if self.buf.len() < self.cap {
            self.buf.push(value);
        } else {
            self.buf[self.head] = value;
            self.head = (self.head + 1) % self.cap;
        }
    }

    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let n = self.buf.len();
        (0..n).map(move |i| &self.buf[(self.head + i) % n])
    }

    pub fn latest(&self) -> Option<&T> {
        if self.buf.is_empty() { return None; }
        let i = (self.head + self.buf.len() - 1) % self.buf.len();
        Some(&self.buf[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_and_keeps_oldest_to_newest_order() {
        let mut rb = RingBuffer::new(3);
        rb.push(1); rb.push(2); rb.push(3); rb.push(4);
        let v: Vec<i32> = rb.iter().copied().collect();
        assert_eq!(v, vec![2, 3, 4]);
        assert_eq!(*rb.latest().unwrap(), 4);
    }

    #[test]
    fn not_full_yields_all_in_order() {
        let mut rb = RingBuffer::new(5);
        rb.push(10); rb.push(20);
        let v: Vec<i32> = rb.iter().copied().collect();
        assert_eq!(v, vec![10, 20]);
        assert_eq!(*rb.latest().unwrap(), 20);
    }

    #[test]
    fn empty_has_no_latest() {
        let rb: RingBuffer<i32> = RingBuffer::new(3);
        assert!(rb.is_empty());
        assert!(rb.latest().is_none());
    }
}
