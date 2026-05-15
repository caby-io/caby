pub const ADJECTIVES: &[&str] = &[
    "Agile", "Bold", "Brave", "Bright", "Calm", "Cheerful", "Clever", "Cosmic", "Cozy", "Crisp",
    "Daring", "Dapper", "Dashing", "Eager", "Fancy", "Fluffy", "Friendly", "Gentle", "Glad",
    "Glossy", "Golden", "Graceful", "Happy", "Hardy", "Helpful", "Hopeful", "Jolly", "Jovial",
    "Kind", "Lively", "Lucky", "Merry", "Mighty", "Mellow", "Modest", "Noble", "Nimble",
    "Peaceful", "Plucky", "Polite", "Proud", "Quick", "Quiet", "Quirky", "Radiant", "Royal",
    "Sunny", "Swift", "Tidy", "Witty",
];

pub const ANIMALS: &[&str] = &[
    "Otter",
    "Badger",
    "Beaver",
    "Bear",
    "Bison",
    "Buffalo",
    "Camel",
    "Cheetah",
    "Cougar",
    "Crane",
    "Deer",
    "Dolphin",
    "Eagle",
    "Elk",
    "Falcon",
    "Ferret",
    "Finch",
    "Fox",
    "Frog",
    "Gazelle",
    "Giraffe",
    "Hawk",
    "Hedgehog",
    "Heron",
    "Horse",
    "Iguana",
    "Jaguar",
    "Koala",
    "Lemur",
    "Lion",
    "Lynx",
    "Magpie",
    "Marmot",
    "Meerkat",
    "Moose",
    "Newt",
    "Ocelot",
    "Octopus",
    "Owl",
    "Panda",
    "Penguin",
    "Puffin",
    "Quail",
    "Rabbit",
    "Raccoon",
    "Salamander",
    "Sparrow",
    "Squirrel",
    "Stoat",
    "Tiger",
];

// FNV-1a 64-bit
pub fn calculate_name(sub: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in sub.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let adj = ADJECTIVES[(hash as usize) % ADJECTIVES.len()];
    let animal = ANIMALS[((hash >> 32) as usize) % ANIMALS.len()];
    format!("{} {}", adj, animal)
}
