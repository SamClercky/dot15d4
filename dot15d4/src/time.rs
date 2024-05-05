//! Time structures.
//!
//! - [`Instant`] is used to represent a point in time.
//! - [`Duration`] is used to represent a duration of time.

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Instant {
    us: i64,
}

impl Instant {
    /// Create a new `Instant` from microseconds since the epoch.
    pub const fn from_us(us: i64) -> Self {
        Self { us }
    }

    /// Returns the point in time as microseconds since the epoch.
    pub const fn as_us(&self) -> i64 {
        self.us
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct Duration(i64);

impl Duration {
    /// Create a new `Duration` from microseconds.
    pub const fn from_us(us: i64) -> Self {
        Self(us)
    }

    /// Returns the duration as microseconds.
    pub const fn as_us(&self) -> i64 {
        self.0
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Duration {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "duration of {}us", self.as_us());
    }
}

impl core::ops::Sub for Instant {
    type Output = Self;

    fn sub(self, rhs: Instant) -> Self::Output {
        Self::from_us(self.as_us() - rhs.as_us())
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self::from_us(self.us - rhs.as_us())
    }
}

impl core::ops::Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self::from_us(self.as_us() - rhs.as_us())
    }
}

impl core::ops::Mul<usize> for Duration {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self::from_us(self.as_us() * rhs as i64)
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self::from_us(self.us + rhs.as_us())
    }
}

impl core::ops::Div<usize> for Duration {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        Self::from_us(self.as_us() / rhs as i64)
    }
}

impl core::ops::Add<Duration> for Duration {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self::from_us(self.as_us() + rhs.as_us())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant() {
        let a = Instant::from_us(100);
        assert_eq!(a.us, 100);
        assert_eq!(a.as_us(), 100);
    }

    #[test]
    fn instant_operations() {
        let a = Instant::from_us(100);
        let b = Instant::from_us(50);
        assert_eq!((a - b).as_us(), 50);
        assert_eq!((a - Duration::from_us(50)).as_us(), 50);
        assert_eq!((a + Duration::from_us(50)).as_us(), 150);
    }

    #[test]
    fn duration() {
        let a = Duration::from_us(100);
        assert_eq!(a.0, 100);
        assert_eq!(a.as_us(), 100);
    }

    #[test]
    fn duration_operations() {
        let a = Duration::from_us(100);
        let b = Duration::from_us(50);
        assert_eq!((a - b).as_us(), 50);
        assert_eq!((a * 2).as_us(), 200);
        assert_eq!((a / 2).as_us(), 50);
        assert_eq!((a + b).as_us(), 150);
    }
}
