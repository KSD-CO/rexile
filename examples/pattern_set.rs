use std::ops::ControlFlow;

use rexile::{PatternSet, SetSearchMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let set = PatternSet::new([
        r"user=([a-z]+)",
        r"code=([0-9]+)",
        r"([a-z]+):([0-9]+);tag0000",
        r"user=alice",
    ])?;
    let text = "code=42 user=alice alice:7;tag0000 user=bob";
    println!(
        "Matched rules: {:?}",
        set.matches(text).iter().collect::<Vec<_>>()
    );
    let mut cache = set.create_cache();
    let _ = set.visit_captures(text, &mut cache, SetSearchMode::All, |hit| {
        println!(
            "rule {} at {:?}: {:?}, group 1: {:?}",
            hit.pattern_id(),
            hit.pos(0),
            hit.get(0),
            hit.get(1),
        );
        ControlFlow::<()>::Continue(())
    });
    Ok(())
}
