//! Zero-cost abstractions for eliminating allocations in hot paths.
//!
//! This module provides zero-allocation alternatives to common patterns
//! that typically require heap allocations.

use core::fmt::{self, Display, Write};
use std::str::FromStr;

// Type aliases to reduce complexity warnings
type DisplayArgs<'display_data> = &'display_data [&'display_data dyn Display];

/// Custom error for `SmallString` operations
#[derive(Debug, Clone, Copy)]
pub struct CapacityError;

/// Zero-allocation string formatter that can be used in const contexts
/// and error messages without requiring heap allocation.
pub struct LazyFormat<F> {
    formatter: F,
}

impl<F> LazyFormat<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    /// Creates a new lazy formatter with the given closure
    #[must_use]
    #[inline]
    pub const fn new(formatter: F) -> Self {
        Self { formatter }
    }
}

impl<F> Display for LazyFormat<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.formatter)(f)
    }
}

/// Zero-allocation string builder for constructing strings without intermediate allocations
pub struct StringBuilder<const N: usize> {
    buffer: [u8; N],
    len: usize,
}

impl<const N: usize> Default for StringBuilder<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> StringBuilder<N> {
    /// Creates a new empty string builder
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; N],
            len: 0,
        }
    }

    /// Appends a string slice to the builder
    ///
    /// # Errors
    ///
    /// Returns `fmt::Error` if adding the string would exceed the buffer capacity
    #[inline]
    pub fn push_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > N {
            return Err(fmt::Error);
        }

        self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }

    /// Appends a single character to the builder
    ///
    /// # Errors
    ///
    /// Returns `fmt::Error` if adding the character would exceed the buffer capacity
    #[inline]
    pub fn push_char(&mut self, c: char) -> Result<(), fmt::Error> {
        let mut buffer = [0u8; 4];
        let s = c.encode_utf8(&mut buffer);
        self.push_str(s)
    }

    /// Returns the string contents as a string slice
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: StringBuilder maintains the invariant that buffer[..len] contains only valid UTF-8
        // We only push valid UTF-8 strings through push_str and push_char methods
        unsafe { std::str::from_utf8_unchecked(&self.buffer[..self.len]) }
    }

    /// Converts the builder into a `String`
    #[must_use]
    #[inline]
    pub fn into_string(self) -> String {
        self.as_str().to_string()
    }

    /// Clears the builder, making it empty
    #[inline]
    pub const fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> Write for StringBuilder<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push_char(c)
    }
}

impl<const N: usize> Display for StringBuilder<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Zero-allocation iterator adapter that avoids collecting into Vec
pub struct ChunkedTargets<I> {
    iter: I,
    chunk_size: usize,
}

impl<I> ChunkedTargets<I> {
    /// Creates a new chunked iterator adapter
    #[must_use]
    #[inline]
    pub const fn new(iter: I, chunk_size: usize) -> Self {
        Self { iter, chunk_size }
    }
}

impl<I, T> Iterator for ChunkedTargets<I>
where
    I: Iterator<Item = T>,
{
    type Item = smallvec::SmallVec<[T; 8]>; // Stack allocated for small chunks

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = smallvec::SmallVec::new();
        for _ in 0..self.chunk_size {
            if let Some(item) = self.iter.next() {
                chunk.push(item);
            } else {
                break;
            }
        }

        if chunk.is_empty() { None } else { Some(chunk) }
    }
}

/// Zero-allocation target display helper
pub struct TargetDisplay<'target_data> {
    target: &'target_data crate::types::Target,
}

impl<'target_data> TargetDisplay<'target_data> {
    /// Creates a new target display helper
    #[must_use]
    #[inline]
    pub const fn new(target: &'target_data crate::types::Target) -> Self {
        Self { target }
    }
}

impl Display for TargetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.target {
            crate::types::Target::Tcp { host, port } => {
                write!(f, "{host}:{port}")
            }
            crate::types::Target::Http { url, .. } => Display::fmt(url, f),
        }
    }
}

/// Zero-allocation error message builder
pub struct ErrorMessage<'message_data> {
    template: &'static str,
    args: DisplayArgs<'message_data>,
}

impl<'message_data> ErrorMessage<'message_data> {
    /// Creates a new error message with template and arguments
    #[must_use]
    #[inline]
    pub const fn new(template: &'static str, args: DisplayArgs<'message_data>) -> Self {
        Self { template, args }
    }
}

impl Display for ErrorMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Simple template replacement - could be enhanced with proper placeholder support
        let mut parts = self.template.split("{}");
        if let Some(first) = parts.next() {
            f.write_str(first)?;
        }

        for (i, part) in parts.enumerate() {
            if let Some(arg) = self.args.get(i) {
                Display::fmt(arg, f)?;
            }
            f.write_str(part)?;
        }

        Ok(())
    }
}

/// Const-friendly port validation using const generics
pub struct ValidatedPort<const MIN: u16, const MAX: u16>(u16);

impl<const MIN: u16, const MAX: u16> ValidatedPort<MIN, MAX> {
    /// Creates a new validated port if it's within the specified range
    #[must_use]
    #[inline]
    pub const fn new(port: u16) -> Option<Self> {
        if port >= MIN && port <= MAX && port != 0 {
            Some(Self(port))
        } else {
            None
        }
    }

    /// Returns the validated port number
    #[must_use]
    #[inline]
    pub const fn get(&self) -> u16 {
        self.0
    }
}

// Common port ranges as type aliases
/// Well-known ports (1-1023) reserved for system services
pub type WellKnownPort = ValidatedPort<1, 1023>;
/// Registered ports (1024-49151) for user applications
pub type RegisteredPort = ValidatedPort<1024, 49151>;
/// Dynamic/private ports (49152-65535) for temporary connections
pub type DynamicPort = ValidatedPort<49152, 65535>;

/// Zero-allocation retry strategy using const generics
pub struct ConstRetryStrategy<const MAX_ATTEMPTS: u32, const INTERVAL_MS: u64>;

impl<const MAX_ATTEMPTS: u32, const INTERVAL_MS: u64> Default
    for ConstRetryStrategy<MAX_ATTEMPTS, INTERVAL_MS>
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_ATTEMPTS: u32, const INTERVAL_MS: u64>
    ConstRetryStrategy<MAX_ATTEMPTS, INTERVAL_MS>
{
    /// Creates a new retry strategy with compile-time configuration
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the maximum number of retry attempts
    #[must_use]
    #[inline]
    pub const fn max_attempts(&self) -> u32 {
        MAX_ATTEMPTS
    }

    /// Returns the retry interval in milliseconds
    #[must_use]
    #[inline]
    pub const fn interval_ms(&self) -> u64 {
        INTERVAL_MS
    }

    /// Determines if another retry attempt should be made
    #[must_use]
    #[inline]
    pub const fn should_retry(&self, attempt: u32) -> bool {
        attempt < MAX_ATTEMPTS
    }
}

/// Stack-allocated string for small strings (avoids heap allocation)
#[derive(Debug, Clone)]
pub struct SmallString<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> Default for SmallString<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> SmallString<N> {
    /// Creates a new empty small string
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }

    /// Attempts to create a small string from a string slice
    ///
    /// Returns `None` if the string is too long for the buffer
    #[must_use]
    #[inline]
    pub fn try_from_str(s: &str) -> Option<Self> {
        if s.len() > N {
            return None;
        }

        let mut result = Self::new();
        let bytes = s.as_bytes();
        result.data[..bytes.len()].copy_from_slice(bytes);
        result.len = bytes.len();
        Some(result)
    }

    /// Returns the string contents as a string slice
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: SmallString maintains the invariant that data[..len] contains only valid UTF-8
        // We only construct SmallString from valid UTF-8 strings in try_from_str and push_str
        unsafe { std::str::from_utf8_unchecked(&self.data[..self.len]) }
    }

    /// Returns the length of the string in bytes
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the string is empty
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends a string slice to the end of this string
    ///
    /// # Errors
    ///
    /// Returns `CapacityError` if adding the string would exceed the buffer capacity
    #[inline]
    pub fn push_str(&mut self, s: &str) -> Result<(), CapacityError> {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > N {
            return Err(CapacityError);
        }

        self.data[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

impl<const N: usize> Display for SmallString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> AsRef<str> for SmallString<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> PartialEq<str> for SmallString<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> PartialEq<&str> for SmallString<N> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const N: usize> FromStr for SmallString<N> {
    type Err = CapacityError;

    fn from_str(s: &str) -> Result<Self, CapacityError> {
        Self::try_from_str(s).ok_or(CapacityError)
    }
}

/// Macro for creating zero-allocation error messages
#[macro_export]
macro_rules! zero_alloc_error {
    ($template:literal $(, $arg:expr)*) => {{
        let args: &[&dyn std::fmt::Display] = &[$(&$arg),*];
        $crate::zero_cost::ErrorMessage::new($template, args)
    }};
}

/// Macro for creating lazy format strings
#[macro_export]
macro_rules! lazy_format {
    ($($arg:tt)*) => {
        $crate::zero_cost::LazyFormat::new(move |f| write!(f, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_builder() {
        let mut builder = StringBuilder::<64>::new();
        builder.push_str("Hello").unwrap();
        builder.push_str(" ").unwrap();
        builder.push_str("World").unwrap();
        assert_eq!(builder.as_str(), "Hello World");
    }

    #[test]
    fn small_string() {
        let s = SmallString::<32>::try_from_str("test").unwrap();
        assert_eq!(s.as_str(), "test");
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn validated_port() {
        let port = WellKnownPort::new(80).unwrap();
        assert_eq!(port.get(), 80);

        assert!(WellKnownPort::new(0).is_none());
        assert!(WellKnownPort::new(1024).is_none());
    }

    #[test]
    fn const_retry_strategy() {
        let strategy = ConstRetryStrategy::<3, 1000>::new();
        assert_eq!(strategy.max_attempts(), 3);
        assert_eq!(strategy.interval_ms(), 1000);
        assert!(strategy.should_retry(2));
        assert!(!strategy.should_retry(3));
    }
}
