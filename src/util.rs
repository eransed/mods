use std::ops::{Add, Div, Mul, Sub};

/// Minimal numeric cast trait to avoid external deps.
/// Convert to/from f64 for statistics.
pub trait NumCast: Copy + Default + PartialOrd {
  fn to_f64(self) -> f64;
}

macro_rules! impl_numcast_for {
    ($($t:ty),+) => {
        $(
            impl NumCast for $t {
                fn to_f64(self) -> f64 { self as f64 }
            }
        )+
    };
}

impl_numcast_for!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// Ring buffer with compile-time capacity N storing numeric type T.
#[derive(Clone, Copy, Debug)]
pub struct ValueWithStats<T, const N: usize>
where
  T: NumCast,
{
  buf: [T; N],
  next: usize,  // next write index
  count: usize, // how many values have been pushed (<= N)
}

#[allow(dead_code)]
impl<T, const N: usize> ValueWithStats<T, N>
where
  T: NumCast,
{
  /// Create a new ring buffer. Capacity is the const generic N.
  pub fn new() -> Self {
    Self { buf: [T::default(); N], next: 0, count: 0 }
  }

  /// Push a new value into the ring buffer.
  pub fn push(&mut self, v: T) {
    if N == 0 {
      return;
    }
    self.buf[self.next] = v;
    self.next = (self.next + 1) % N;
    if self.count < N {
      self.count += 1;
    }
  }

  /// Number of values that have been pushed (up to capacity).
  pub fn size(&self) -> usize {
    self.count
  }

  /// Capacity (part of the type).
  pub const fn capacity(&self) -> usize {
    N
  }

  /// Get last pushed value if any.
  pub fn last(&self) -> Option<T> {
    if self.count == 0 || N == 0 {
      return None;
    }
    let idx = if self.next == 0 { N - 1 } else { self.next - 1 };
    Some(self.buf[idx])
  }

  fn iter_last_n_values(&self, n: usize) -> Vec<T> {
    let count = self.count.min(n);
    let mut v = Vec::with_capacity(count);
    if count == 0 {
      return v;
    }

    let mut idx = if self.count < N { self.count - count } else { (self.next + N - count) % N };

    for _ in 0..count {
      v.push(self.buf[idx]);
      idx = (idx + 1) % N;
    }

    v
  }

  /// Minimum value (None if empty).
  pub fn min(&self) -> Option<T> {
    self.min_last_n(self.count)
  }

  /// Maximum value (None if empty).
  pub fn max(&self) -> Option<T> {
    self.max_last_n(self.count)
  }

  /// Mean as f64 (None if empty).
  pub fn mean(&self) -> Option<f64> {
    self.mean_last_n(self.count)
  }

  /// Population standard deviation (None if empty).
  pub fn standard_deviation(&self) -> Option<f64> {
    self.standard_deviation_last_n(self.count)
  }

  /// Median as f64 (None if empty).
  pub fn median(&self) -> Option<f64> {
    self.median_last_n(self.count)
  }

  /// Range = max - min (None if empty).
  pub fn range(&self) -> Option<f64> {
    self.range_last_n(self.count)
  }

  /// Minimum over the last n pushed values.
  pub fn min_last_n(&self, n: usize) -> Option<T> {
    let vals = self.iter_last_n_values(n);
    if vals.is_empty() {
      return None;
    }
    let mut m = vals[0];
    for &v in &vals[1..] {
      if v < m {
        m = v;
      }
    }
    Some(m)
  }

  /// Maximum over the last n pushed values.
  pub fn max_last_n(&self, n: usize) -> Option<T> {
    let vals = self.iter_last_n_values(n);
    if vals.is_empty() {
      return None;
    }
    let mut m = vals[0];
    for &v in &vals[1..] {
      if v > m {
        m = v;
      }
    }
    Some(m)
  }

  /// Mean over the last n pushed values.
  pub fn mean_last_n(&self, n: usize) -> Option<f64> {
    let vals = self.iter_last_n_values(n);
    if vals.is_empty() {
      return None;
    }
    let sum: f64 = vals.iter().map(|x| x.to_f64()).sum();
    Some(sum / (vals.len() as f64))
  }

  /// Population standard deviation over the last n pushed values.
  pub fn standard_deviation_last_n(&self, n: usize) -> Option<f64> {
    let vals = self.iter_last_n_values(n);
    if vals.is_empty() {
      return None;
    }
    let mean = vals.iter().map(|x| x.to_f64()).sum::<f64>() / (vals.len() as f64);
    let var = vals
      .iter()
      .map(|x| {
        let diff = x.to_f64() - mean;
        diff * diff
      })
      .sum::<f64>()
      / (vals.len() as f64);
    Some(var.sqrt())
  }

  /// Median over the last n pushed values.
  pub fn median_last_n(&self, n: usize) -> Option<f64> {
    let mut vals = self.iter_last_n_values(n).into_iter().map(|x| x.to_f64()).collect::<Vec<f64>>();
    if vals.is_empty() {
      return None;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = vals.len() / 2;
    if vals.len() % 2 == 1 { Some(vals[mid]) } else { Some((vals[mid - 1] + vals[mid]) / 2.0) }
  }

  /// Range = max - min over the last n pushed values.
  pub fn range_last_n(&self, n: usize) -> Option<f64> {
    match (self.min_last_n(n), self.max_last_n(n)) {
      (Some(mi), Some(ma)) => Some(ma.to_f64() - mi.to_f64()),
      _ => None,
    }
  }
}

impl<T, const N: usize> Add<T> for ValueWithStats<T, N>
where
  T: NumCast + Add<Output = T>,
{
  type Output = T;
  fn add(self, rhs: T) -> T {
    let l = self.last().unwrap_or_default();
    l + rhs
  }
}
impl<T, const N: usize> Sub<T> for ValueWithStats<T, N>
where
  T: NumCast + Sub<Output = T>,
{
  type Output = T;
  fn sub(self, rhs: T) -> T {
    let l = self.last().unwrap_or_default();
    l - rhs
  }
}
impl<T, const N: usize> Mul<T> for ValueWithStats<T, N>
where
  T: NumCast + Mul<Output = T>,
{
  type Output = T;
  fn mul(self, rhs: T) -> T {
    let l = self.last().unwrap_or_default();
    l * rhs
  }
}
impl<T, const N: usize> Div<T> for ValueWithStats<T, N>
where
  T: NumCast + Div<Output = T>,
{
  type Output = T;
  fn div(self, rhs: T) -> T {
    let l = self.last().unwrap_or_default();
    l / rhs
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn push_and_size() {
    let mut v: ValueWithStats<f64, 3> = ValueWithStats::new();
    assert_eq!(v.size(), 0);
    assert_eq!(v.capacity(), 3);
    v.push(1.0);
    assert_eq!(v.size(), 1);
    v.push(2.0);
    v.push(3.0);
    assert_eq!(v.size(), 3);
    v.push(4.0);
    // capacity reached, size stays at capacity
    assert_eq!(v.size(), 3);
    // last should be 4.0 (ring behaviour)
    assert_eq!(v.last(), Some(4.0));
  }

  #[test]
  fn min_max_range() {
    let mut v: ValueWithStats<i32, 5> = ValueWithStats::new();
    v.push(10);
    v.push(3);
    v.push(7);
    assert_eq!(v.min(), Some(3));
    assert_eq!(v.max(), Some(10));
    assert_eq!(v.range(), Some(7.0));
  }

  #[test]
  fn last_n_variants_partial_and_wrapped() {
    let mut v: ValueWithStats<i32, 4> = ValueWithStats::new();
    v.push(5);
    v.push(1);
    v.push(3);

    assert_eq!(v.min_last_n(2), Some(1));
    assert_eq!(v.max_last_n(2), Some(3));
    assert_eq!(v.mean_last_n(2), Some(2.0));
    assert!((v.standard_deviation_last_n(2).unwrap() - 1.0).abs() < 1e-12);
    assert_eq!(v.median_last_n(2), Some(2.0));
    assert_eq!(v.range_last_n(2), Some(2.0));

    assert_eq!(v.min_last_n(5), Some(1));
    assert_eq!(v.max_last_n(5), Some(5));
    assert_eq!(v.mean_last_n(5), Some(3.0));
    assert!((v.standard_deviation_last_n(5).unwrap() - (8.0f64 / 3.0).sqrt()).abs() < 1e-12);
    assert_eq!(v.median_last_n(5), Some(3.0));
    assert_eq!(v.range_last_n(5), Some(4.0));

    v.push(9);
    v.push(2);
    assert_eq!(v.last(), Some(2));
    assert_eq!(v.min_last_n(3), Some(2));
    assert_eq!(v.max_last_n(3), Some(9));
    assert_eq!(v.mean_last_n(3), Some((3 + 9 + 2) as f64 / 3.0));
    assert_eq!(v.median_last_n(3), Some(3.0));
    assert_eq!(v.range_last_n(3), Some(7.0));
  }

  #[test]
  fn mean_and_stddev() {
    let mut v: ValueWithStats<f64, 4> = ValueWithStats::new();
    v.push(2.0);
    v.push(4.0);
    v.push(4.0);
    v.push(4.0);
    // mean = 3.5
    let m = v.mean().unwrap();
    assert!((m - 3.5).abs() < 1e-12);
    // population stddev = sqrt(((1.5^2 + 0.5^2 + 0.5^2 + 0.5^2)/4)) = sqrt(0.75)
    let sd = v.standard_deviation().unwrap();
    assert!((sd - (0.75f64).sqrt()).abs() < 1e-12);
  }

  #[test]
  fn median_even_odd() {
    let mut v: ValueWithStats<f64, 5> = ValueWithStats::new();
    v.push(5.0);
    v.push(1.0);
    v.push(3.0);
    assert_eq!(v.median().unwrap(), 3.0); // odd
    v.push(2.0);
    // values [5,1,3,2] -> sorted [1,2,3,5] median = (2+3)/2 = 2.5
    assert!((v.median().unwrap() - 2.5).abs() < 1e-12);
  }

  #[test]
  fn arithmetic_ops_use_last() {
    let mut v: ValueWithStats<i32, 2> = ValueWithStats::new();
    v.push(10);
    assert_eq!(v + 5, 15);
    v.push(7);
    assert_eq!(v - 2, 5); // last is 7
    assert_eq!(v * 2, 14);
    assert_eq!(v / 7, 1);
  }
}
