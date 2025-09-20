use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

pub struct Events<E> {
    next: Option<E>,
    queue: VecDeque<E>,
}

impl<E> Events<E> {
    pub fn queue(&mut self, event: E) {
        self.queue.push_back(event);
    }

    pub fn step(&mut self) -> Option<E> {
        self.queue
            .pop_back()
            .and_then(|event| self.next.replace(event))
    }

    pub fn step_or(&mut self, default: E) -> Option<E> {
        self.next.replace(self.queue.pop_back().unwrap_or(default))
    }

    pub fn step_or_else<F: FnOnce() -> E>(&mut self, f: F) -> Option<E> {
        self.next.replace(self.queue.pop_back().unwrap_or_else(f))
    }
}

impl<E> Deref for Events<E> {
    type Target = Option<E>;

    fn deref(&self) -> &Self::Target {
        &self.next
    }
}

impl<E> DerefMut for Events<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.next
    }
}
