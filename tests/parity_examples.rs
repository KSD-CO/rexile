use rexile;
use regex::Regex;

#[test]
fn anchors_and_basic() {
    let pat = "^foo\\d+bar$";
    let txt_ok = "foo123bar";
    let txt_bad = "xfoo123bar";

    let r = Regex::new(pat).unwrap();
    assert_eq!(r.is_match(txt_ok), true);
    assert_eq!(r.is_match(txt_bad), false);

    assert_eq!(rexile::is_match(pat, txt_ok).unwrap(), true);
    assert_eq!(rexile::is_match(pat, txt_bad).unwrap(), false);
}

#[test]
fn named_captures_and_find() {
    let pat = "(?P<name>user)\\s*=(?P<id>\\d+)";
    let txt = "name user =123 extra";

    let r = Regex::new(pat).unwrap();
    let caps = r.captures(txt).unwrap();
    assert_eq!(caps.name("name").unwrap().as_str(), "user");
    assert_eq!(caps.name("id").unwrap().as_str(), "123");

    // rexile find returns offsets; use regex to check substring
    let pos = rexile::find(pat, txt).unwrap().unwrap();
    let (s, e) = pos;
    assert_eq!(&txt[s..e], "user =123");

    // static retrieval should still match
    let sref = rexile::get_regex_static(pat).unwrap();
    let caps2 = sref.captures(txt).unwrap();
    assert_eq!(caps2.name("id").unwrap().as_str(), "123");
}

#[test]
fn alternation_and_unicode() {
    let pat = "gr(a|e)y|挨个"; // alternation and a chinese word
    let txt1 = "gray";
    let txt2 = "挨个";

    let r = Regex::new(pat).unwrap();
    assert!(r.is_match(txt1));
    assert!(r.is_match(txt2));

    assert!(rexile::is_match(pat, txt1).unwrap());
    assert!(rexile::is_match(pat, txt2).unwrap());
}

#[test]
fn invalid_pattern_error() {
    // an invalid pattern should return an error both for regex and rexile
    let bad = "(unclosed";
    assert!(Regex::new(bad).is_err());
    assert!(rexile::get_regex(bad).is_err());
    assert!(rexile::get_regex_static(bad).is_err());
}
