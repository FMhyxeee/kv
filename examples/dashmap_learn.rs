use dashmap::DashMap;

fn main() {
    let reviews = DashMap::new();
    reviews.insert("Veloren", "What a fantastic game!");
    println!("{reviews:?}");

    let mappings = DashMap::with_capacity(2);
    mappings.insert(2, 4);
    mappings.insert(8, 16);
    println!("{mappings:?}")
}
