//! ReXile
//!
//! Minimal skeleton for the ReXile project: a small wrapper for cached regexes
//! and an ergonomic API useful during migration from the legacy `regex` usages.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;

// Primary mutable cache for owned Regex values. This keeps a single copy per
// pattern and is suitable for the common case where callers want an owned
// Regex handle or quick lookups.
static CACHE: Lazy<Mutex<HashMap<String, Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Secondary static cache for callers that need a `'static` Regex reference.
// WARNING: entries inserted here intentionally leak memory (Box::leak) to
// provide a true `'static` lifetime. This is acceptable for long-running
// programs where regex patterns are bootstrap-time constants; if you need
// GC'd behavior, add an explicit removal API.
static STATIC_CACHE: Lazy<Mutex<HashMap<String, &'static Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Compile or retrieve a cached owned Regex for `pat`.
pub fn get_regex(pat: &str) -> Result<Regex, regex::Error> {
    let mut lock = CACHE.lock().unwrap();
    if let Some(r) = lock.get(pat) {
        return Ok(r.clone());
    }
    let r = Regex::new(pat)?;
    // Insert the owned Regex into cache and return a clone for the caller.
    lock.insert(pat.to_owned(), r.clone());
    Ok(r)
}

/// Return whether `text` matches `pat`, using the cache when possible.
///
/// This helper avoids allocating Regex handles on the hot path: it looks up a
/// cached Regex and calls `is_match` directly, only compiling once when the
/// pattern is first seen.
pub fn is_match(pat: &str, text: &str) -> Result<bool, regex::Error> {
    let mut lock = CACHE.lock().unwrap();
    if let Some(r) = lock.get(pat) {
        return Ok(r.is_match(text));
    }
    let r = Regex::new(pat)?;
    let is = r.is_match(text);
    lock.insert(pat.to_owned(), r);
    Ok(is)
}

/// Find the first match for `pat` in `text`, returning the start/end offsets.
pub fn find(pat: &str, text: &str) -> Result<Option<(usize, usize)>, regex::Error> {
    let mut lock = CACHE.lock().unwrap();
    if let Some(r) = lock.get(pat) {
        return Ok(r.find(text).map(|m| (m.start(), m.end())));
    }
    let r = Regex::new(pat)?;
    let res = r.find(text).map(|m| (m.start(), m.end()));
    lock.insert(pat.to_owned(), r);
    Ok(res)
}

/// Compile or retrieve a `'static` Regex reference for `pat`.
///
/// This creates a leaked boxed Regex on first use and returns a reference with
/// a `'static` lifetime. Use sparingly: leaking is intentional to avoid
/// lifetime/ownership complexity during migration. Future work may replace this
/// with a proper global registry that doesn't rely on leaks.
pub fn get_regex_static(pat: &str) -> Result<&'static Regex, regex::Error> {
    let mut lock = STATIC_CACHE.lock().unwrap();
    if let Some(r) = lock.get(pat) {
        return Ok(*r);
    }
    let r = Regex::new(pat)?;
    let leaked: &'static Regex = Box::leak(Box::new(r));
    lock.insert(pat.to_owned(), leaked);
    Ok(leaked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_basic() {
        let r1 = get_regex("^foo").unwrap();
        let r2 = get_regex("^foo").unwrap();
        assert_eq!(r1.as_str(), r2.as_str());
    }

    #[test]
    fn is_match_helper() {
        assert!(is_match("^rule\\s+", "rule foo").unwrap());
        assert!(!is_match("^rule\\s+", "foo rule").unwrap());
    }

    #[test]
    fn find_helper_and_static() {
        let txt = "before rule 123 after";
        let pos = find("rule\\s+\\d+", txt).unwrap();
        assert!(pos.is_some());
        let (s, e) = pos.unwrap();
        assert_eq!(&txt[s..e], "rule 123");

        // static retrieval should succeed and return a stable reference
        let sref = get_regex_static("rule\\s+\\d+").unwrap();
        assert!(sref.is_match(txt));
    }
}
