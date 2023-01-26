use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Cursor<T> {
    index: AtomicUsize,
    data: Vec<T>,
}

impl<T> Default for Cursor<T> {
    fn default() -> Self {
        Self {
            index: Default::default(),
            data: Default::default(),
        }
    }
}

impl<T> Cursor<T> {
    pub fn new(t: impl IntoIterator<Item = T>) -> Self {
        Self {
            index: AtomicUsize::new(0),
            data: t.into_iter().collect(),
        }
    }

    pub fn goto(&self, index: usize) -> bool {
        if index >= self.data.len() {
            false
        } else {
            let index = usize::min(index, self.data.len() - 1);
            self.index.store(index, Ordering::Release);
            true
        }
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    fn constrain(&self) {
        let index = self.index.load(Ordering::Acquire);
        self.index.store(
            usize::min(self.data.len().saturating_sub(1), index),
            Ordering::Release,
        );
    }

    pub fn index(&self) -> usize {
        self.index.load(Ordering::Acquire)
    }

    pub fn next_mut(&mut self) -> &mut T {
        self.goto(self.index.load(Ordering::Acquire) + 1);
        self.get_mut()
    }

    pub fn prev_mut(&mut self) -> &mut T {
        self.goto(self.index.load(Ordering::Acquire).saturating_sub(1));
        self.get_mut()
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data[self.index.load(Ordering::Acquire)]
    }

    pub fn get_at_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    pub fn next(&self) -> &T {
        self.goto(self.index.load(Ordering::Acquire) + 1);
        self.get()
    }

    pub fn prev(&self) -> &T {
        self.goto(self.index.load(Ordering::Acquire).saturating_sub(1));
        self.get()
    }

    pub fn get(&self) -> &T {
        &self.data[self.index.load(Ordering::Acquire)]
    }

    pub fn get_at(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
        self.constrain();
    }

    pub fn pop(&mut self) -> Option<T> {
        let v = self.data.pop();
        self.constrain();
        v
    }

    pub fn remove(&mut self, index: usize) {
        self.data.remove(index);
        self.constrain();
    }
}
