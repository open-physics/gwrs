use std::cmp::{Ord, Ordering, max, min};
use std::ops::{BitAnd, BitOr, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment<T: PartialOrd + Copy> {
    pub start: T,
    pub end: T,
}

impl<T: PartialOrd + Copy> Segment<T> {
    pub fn new(a: T, b: T) -> Self {
        if a <= b {
            Self { start: a, end: b }
        } else {
            Self { start: b, end: a }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, value: T) -> bool {
        self.start <= value && value < self.end
    }

    pub fn contains_segment(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }
}

// Intersection: Segment(0, 10) & Segment(5, 15) == Segment(5, 10)
impl<T: PartialOrd + Copy + Ord> BitAnd for Segment<T> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        let start = max(self.start, rhs.start);
        let end = min(self.end, rhs.end);
        Segment::new(start, end)
    }
}

// Union: Segment(0, 10) | Segment(5, 15) == Segment(0, 15)
impl<T: PartialOrd + Copy + Ord> BitOr for Segment<T> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        let start = min(self.start, rhs.start);
        let end = max(self.end, rhs.end);
        Segment::new(start, end)
    }
}

// Difference: Segment(0, 10) - Segment(5, 15) == Segment(0, 5)
impl<T: PartialOrd + Copy> Sub for Segment<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        if rhs.contains(self.start) && rhs.contains(self.end) {
            Segment::new(self.start, self.start)
        } else if rhs.contains(self.start) {
            Segment::new(rhs.end, self.end)
        } else if rhs.contains(self.end) {
            Segment::new(self.start, rhs.start)
        } else {
            self // no overlap
        }
    }
}

// Implement ordering based on lexical order of start, then end
impl<T: PartialOrd + Ord + Copy> PartialOrd for Segment<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<T: Ord + Copy> Ord for Segment<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start).then(self.end.cmp(&other.end))
    }
}
