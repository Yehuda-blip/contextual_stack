use std::{cmp::Reverse, collections::BinaryHeap};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot<const RESERVE: usize> { s: usize }

impl<const RESERVE: usize> Slot<RESERVE> {
    pub fn to_usize(&self) -> usize {
        self.s + RESERVE
    }
}

#[derive(Debug)]
pub struct Slots<const RESERVE: usize> {
    counter: usize,
    holes: BinaryHeap<Reverse<usize>>,
}

impl<const RESERVE: usize> Slots<RESERVE> {
    pub fn new() -> Self {
        Slots {
            counter: RESERVE,
            holes: BinaryHeap::new(),
        }
    }

    pub fn allocate(&mut self) -> Slot<RESERVE> {
        if let Some(Reverse(slot)) = self.holes.pop() {
            Slot { s: slot }
        } else {
            let slot = self.counter;
            self.counter += 1;
            Slot { s: slot }
        }
    }

    pub fn deallocate(&mut self, slot: Slot<RESERVE>) {
        let Slot { s: slot } = slot;
        if self.counter == slot + 1 {
            self.counter -= 1;
        } else {
            self.holes.push(Reverse(slot));
        }
    }

    pub const fn reserved(res: usize) -> Option<Slot<RESERVE>> {
        if res < RESERVE {
            Some(Slot { s: res })
        } else {
            None
        }
    }
}
