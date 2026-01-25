use rexile::Pattern;

fn main() {
    let pattern = Pattern::new(r"\s+").unwrap();
    println!("Pattern: {:?}", pattern);
}
