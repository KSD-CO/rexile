use rexile::Pattern;

fn main() {
    println!("Testing simple 'a' pattern:");
    let pattern = Pattern::new("a").unwrap();
    println!("  Pattern: a");
    println!("  Text: aaaaa");
    println!("  Result: {:?}", pattern.find("aaaaa"));

    println!("\nTesting 'a+' pattern:");
    let pattern = Pattern::new("a+").unwrap();
    println!("  Pattern: a+");
    println!("  Text: aaaaa");
    println!("  Result: {:?}", pattern.find("aaaaa"));

    println!("\nTesting 'a{{2}}' pattern:");
    let pattern = Pattern::new("a{2}").unwrap();
    println!("  Pattern: a{{2}}");
    println!("  Text: aaaaa");
    println!("  Result: {:?}", pattern.find("aaaaa"));

    println!("\nTesting 'a{{2,}}' pattern:");
    let pattern = Pattern::new("a{2,}").unwrap();
    println!("  Pattern: a{{2,}}");
    println!("  Text: aaaaa");
    println!("  Result: {:?}", pattern.find("aaaaa"));

    println!("\nTesting 'a{{2,4}}' pattern:");
    let pattern = Pattern::new("a{2,4}").unwrap();
    println!("  Pattern: a{{2,4}}");
    println!("  Text: aaaaa");
    println!("  Result: {:?}", pattern.find("aaaaa"));
}
