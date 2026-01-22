// NOTE: This example demonstrates PLANNED future features (capture groups, replace, split)
// that are not yet implemented in ReXile. It's kept here as a design reference.
//
// Current ReXile API supports:
// - Pattern matching: is_match(), find(), find_all()
// - Anchors: ^, $
// - Character classes: [abc], \d, \w, \s
// - Quantifiers: *, +, ?, {n}, {n,}, {n,m}
// - Alternation: a|b
// - Word boundaries: \b, \B
//
// For a working example, see examples/literal_search_demo.rs

fn main() {
    println!("This example demonstrates FUTURE features not yet implemented:");
    println!("- Capture groups: (\\w+)");
    println!("- captures_iter()");
    println!("- replace_all()");
    println!("- split()");
    println!();
    println!("See examples/literal_search_demo.rs for current working features.");
    
    // Example of what WILL work when implemented:
    /*
    let pat = "(\\w+)=(\\d+)";
    let text = "a=1 b=2 c=3";

    println!("pattern: {} | text: {}", pat, text);

    // captures_iter-like behavior
    let r = rexile::get_pattern(pat).unwrap();
    for caps in r.captures_iter(text) {
        println!("capture: full='{}' g1='{}' g2='{}'", &caps[0], &caps[1], &caps[2]);
    }

    // replace (using captured groups)
    let replaced = r.replace_all(text, "$1:[$2]");
    println!("replaced: {}", replaced);

    // split
    let split: Vec<&str> = r.split(text).collect();
    println!("split parts: {:?}", split);
    */
}
