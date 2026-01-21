fn main() {
    let pat = "(\\w+)=(\\d+)";
    let text = "a=1 b=2 c=3";

    println!("pattern: {} | text: {}", pat, text);

    // captures_iter-like behavior using the static regex
    let r = rexile::get_regex(pat).unwrap();
    for caps in r.captures_iter(text) {
        println!("capture: full='{}' g1='{}' g2='{}'", &caps[0], &caps[1], &caps[2]);
    }

    // replace (using regex::Regex on the compiled pattern)
    let replaced = r.replace_all(text, "$1:[$2]");
    println!("replaced: {}", replaced);

    // split
    let split: Vec<&str> = r.split(text).collect();
    println!("split parts: {:?}", split);
}
