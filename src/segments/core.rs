use std::cmp::PartialOrd;
use std::ops::{BitAnd, BitOr, Sub};

/// A struct defining a semi-open interval `[start, end)`.
///
/// Each `Segment` represents the range of values in a given interval, with
/// general arithmetic supported for combining/comparing overlapping segments.
///
/// The `Segment` is generic over type `T`, which must implement `PartialOrd`
/// for comparisons and `Copy` for efficient value handling.
///
/// # Examples
/// ```
/// use gwrs::segments::core::Segment;
/// // Creating segments with inverted ranges
/// let s1 = Segment::new(10.0, 5.0);
/// assert_eq!(s1.start(), 5.0);
/// assert_eq!(s1.end(), 10.0);
/// // Returning start and end values
/// let s2 = Segment::new(0.0, 20.0);
/// let s3 = Segment::new(5.0, 15.0);
/// assert_eq!(s2.start(), 0.0);
/// assert_eq!(s2.end(), 20.0);
/// assert_eq!(s3.start(), 5.0);
/// assert_eq!(s3.end(), 15.0);
/// // Checking containment
/// assert!(s2.contains(&Segment::new(1.0, 2.0)));
/// assert!(s2.contains(&s3));
/// assert!(s2.contains(&Segment::new(0.0, 20.0))); // Contains itself
/// assert!(s2.contains(&s2)); // Contains itself
/// assert!(Segment::new(0.0, 0.0).contains(&Segment::new(0.0, 0.0))); // Empty contains empty
/// // Debug representation
/// assert_eq!(format!("{:?}", s2), "Segment { start: 0.0, end: 20.0 }");
/// // Checking if segments are empty
/// assert!(!s1.is_empty());
/// assert!(Segment::new(0.0, 0.0).is_empty());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Segment {
    pub start: f64,
    pub end: f64,
}

impl Segment {
    /// Creates a new `Segment` instance.
    ///
    /// If `start` is greater than `end`, they are swapped to ensure
    /// the segment is always represented as `[min_value, max_value)`.
    pub fn new(start: f64, end: f64) -> Self {
        if start > end {
            Segment {
                start: end,
                end: start,
            }
        } else {
            Segment { start, end }
        }
    }

    /// Returns the start value of this segment.
    pub fn start(&self) -> f64 {
        self.start
    }

    /// Returns the end value of this segment.
    pub fn end(&self) -> f64 {
        self.end
    }

    /// Checks if this segment contains another segment.
    pub fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Checks if the segment is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

// Intersection: Segment::new(0, 10) & Segment::new(5, 15) == Segment::new(5, 10)
// Implements the intersection (`&`) operator.
impl BitAnd for Segment {
    /// Returns a new `Segment` representing the intersection of `self` and `rhs`.
    /// # Examples
    /// ```rust
    /// use std::ops::BitAnd;
    /// use gwrs::segments::core::Segment;
    /// let s1 = Segment::new(0.0, 10.0);
    /// let s2 = Segment::new(5.0, 15.0);
    /// assert_eq!(s1 & s2, Segment::new(5.0, 10.0));
    /// ```
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        let start = self.start.max(rhs.start);
        let end = self.end.min(rhs.end);
        // If the segments do not overlap, return an empty segment
        if start >= end {
            Segment::new(start, start) // Empty segment
        } else {
            Segment::new(start, end)
        }
    }
}
// Union: Segment::new(0, 10) | Segment::new(5, 15) == Segment::new(0, 15)
// Implements the union (`|`) operator.
impl BitOr for Segment {
    /// Returns a new `Segment` representing the union of `self` and `rhs`.
    /// # Examples
    /// ```rust
    /// use std::ops::BitOr;
    /// use gwrs::segments::core::Segment;
    /// let s1 = Segment::new(0.0, 10.0);
    /// let s2 = Segment::new(5.0, 15.0);
    /// assert_eq!(s1 | s2, Segment::new(0.0, 15.0));
    /// ```
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        let start = self.start.min(rhs.start);
        let end = self.end.max(rhs.end);
        // If the segments do not overlap, return a segment covering both ranges
        Segment::new(start, end)
    }
}
// Difference: Segment::new(0, 10) - Segment::new(5, 15) == Segment::new(0, 5)
// Implements the difference (`-`) operator.
impl Sub for Segment {
    /// Returns a new `Segment` representing the part of `self` that is not covered
    /// by `rhs`.
    /// # Examples
    /// ```rust
    /// use std::ops::Sub;
    /// use gwrs::segments::core::Segment;
    /// let s1 = Segment::new(0.0, 10.0);
    /// let s2 = Segment::new(5.0, 15.0);
    /// assert_eq!(s1 - s2, Segment::new(0.0, 5.0));
    /// let s3 = Segment::new(5.0, 15.0);
    /// let s4 = Segment::new(0.0, 10.0);
    /// assert_eq!(s3 - s4, Segment::new(10.0, 15.0));
    /// let s5 = Segment::new(0.0, 10.0);
    /// let s6 = Segment::new(2.0, 8.0);
    /// assert_eq!(s5 - s6, Segment::new(0.0, 2.0));
    /// ```
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        // If self is entirely before or after rhs, no change
        if self.end <= rhs.start || rhs.end <= self.start {
            return self;
        }
        // If rhs fully contains self, result is an empty segment
        if rhs.start <= self.start && self.end <= rhs.end {
            return Segment::new(self.start, self.start); // Empty segment
        }
        // If self fully contains rhs
        // (e.g., (0,10) - (5,8) -> (0,5))
        // Or if rhs starts within self and ends after self's end
        // (e.g., (0,10) - (5,15) -> (0,5))
        if self.start < rhs.start && rhs.start < self.end {
            return Segment::new(self.start, rhs.start);
        }
        // If rhs starts before self's start and ends within self (e.g., (5,15) - (0,10) -> (10,15))
        if rhs.start < self.start && self.start < rhs.end && rhs.end < self.end {
            return Segment::new(rhs.end, self.end);
        }
        Segment::new(self.start, self.start) // Empty segment
    }
}

/// Unit tests to verify functionality
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_segment() {
        assert_eq!(
            Segment::new(0.0, 10.0),
            Segment {
                start: 0.0,
                end: 10.0
            }
        );
        assert_eq!(
            Segment::new(10.0, 0.0),
            Segment {
                start: 0.0,
                end: 10.0
            }
        );
        assert_eq!(
            Segment::new(5.0, 5.0),
            Segment {
                start: 5.0,
                end: 5.0
            }
        );
    }

    #[test]
    fn test_start_end_properties() {
        let s = Segment::new(1.0, 5.0);
        assert_eq!(s.start(), 1.0);
        assert_eq!(s.end(), 5.0);
    }
    #[test]
    fn test_contains() {
        assert!(Segment::new(0.0, 10.0).contains(&Segment::new(1.0, 2.0)));
        assert!(!Segment::new(0.0, 10.0).contains(&Segment::new(1.0, 11.0)));
        assert!(!Segment::new(0.0, 10.0).contains(&Segment::new(-1.0, 2.0)));
        assert!(Segment::new(0.0, 10.0).contains(&Segment::new(0.0, 10.0))); // Contains itself
        assert!(Segment::new(0.0, 0.0).contains(&Segment::new(0.0, 0.0))); // Empty contains empty
    }

    #[test]
    fn test_is_empty() {
        assert!(!Segment::new(0.0, 1.0).is_empty());
        assert!(Segment::new(0.0, 0.0).is_empty());
        assert!(Segment::new(5.0, 5.0).is_empty());
    }

    #[test]
    fn test_debug_repr() {
        let s = Segment::new(1.0, 5.0);
        assert_eq!(format!("{:?}", s), "Segment { start: 1.0, end: 5.0 }");
    }

    // Intersection: Segment::new(0, 10) & Segment::new(5, 15) == Segment::new(5, 10)
    #[test]
    fn test_bitand_intersection() {
        let s1 = Segment::new(0.0, 10.0);
        let s2 = Segment::new(5.0, 15.0);
        assert_eq!(s1 & s2, Segment::new(5.0, 10.0));
    }
    // Union: Segment::new(0, 10) | Segment::new(5, 15) == Segment::new(0, 15)
    // Union: Segment::new(0, 5) | Segment::new(10, 15) == Segment::new(0, 15)
    #[test]
    fn test_bitor_union() {
        let s1 = Segment::new(0.0, 10.0);
        let s2 = Segment::new(5.0, 15.0);
        assert_eq!(s1 | s2, Segment::new(0.0, 15.0));
        let s3 = Segment::new(0.0, 5.0);
        let s4 = Segment::new(10.0, 15.0);
        assert_eq!(s3 | s4, Segment::new(0.0, 15.0));
    }
    // Difference: Segment::new(0, 10) - Segment::new(5, 15) == Segment::new(0, 5)
    // Difference: Segment::new(5, 15) - Segment::new(0, 10) == Segment::new(10, 15)
    // Difference: Segment::new(0, 10) - Segment::new(2, 8) == Segment::new(0, 2)
    /// Implements the difference (`-`) operator.
    #[test]
    fn test_sub_difference() {
        assert_eq!(
            Segment::new(0.0, 10.0) - Segment::new(5.0, 15.0),
            Segment::new(0.0, 5.0)
        );
        // Overlap from left
        assert_eq!(
            Segment::new(5.0, 15.0) - Segment::new(0.0, 10.0),
            Segment::new(10.0, 15.0)
        );
        assert_eq!(
            Segment::new(0.0, 10.0) - Segment::new(2.0, 8.0),
            Segment::new(0.0, 2.0)
        );
        // No overlap (self before rhs)
        assert_eq!(
            Segment::new(0.0, 5.0) - Segment::new(10.0, 15.0),
            Segment::new(0.0, 5.0)
        );

        // No overlap (self after rhs)
        assert_eq!(
            Segment::new(10.0, 15.0) - Segment::new(0.0, 5.0),
            Segment::new(10.0, 15.0)
        );

        // rhs fully contains self
        assert_eq!(
            Segment::new(2.0, 8.0) - Segment::new(0.0, 10.0),
            Segment::new(2.0, 2.0)
        ); // Empty segment

        // self fully contains self (arbitrary choice: returns first part)
        assert_eq!(
            Segment::new(0.0, 10.0) - Segment::new(2.0, 8.0),
            Segment::new(0.0, 2.0)
        );
    }
    // Less than comparison (compares start, then end)
    #[test]
    fn test_partial_ord_less_than() {
        assert!(Segment::new(0.0, 10.0) < Segment::new(5.0, 15.0));
        assert!(Segment::new(0.0, 10.0) < Segment::new(5.0, 10.0));
        assert!(Segment::new(0.0, 10.0) < Segment::new(5.0, 8.0));
        assert!(Segment::new(5.0, 10.0) < Segment::new(5.0, 15.0));
        assert!(Segment::new(6.0, 10.0) > Segment::new(5.0, 15.0));
        assert!(Segment::new(0.0, 10.0) < Segment::new(0.0, 15.0)); // Same start, different end
        assert!(Segment::new(0.0, 10.0) == Segment::new(0.0, 10.0)); // Equal
    }
}
