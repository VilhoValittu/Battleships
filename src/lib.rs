//! Pelien logiikka ilman ikkunaa, piirtoa tai syötettä.

pub mod arpa;
pub mod laivasto;
pub mod lukulauta;
pub mod ruutu;
pub mod vastustaja;

pub use arpa::Arpa;
pub use laivasto::{Laiva, Laivasto, Laukaustulos};
pub use ruutu::{Ruudut, Ruutu};
pub use vastustaja::Vaikeus;

pub const KOKO: usize = 10;
pub const SARAKKEET: &str = "ABCDEFGHIJ";
pub const LAIVAT: [(&str, usize); 5] = [
    ("Lentotukialus", 5),
    ("Taistelulaiva", 4),
    ("Hävittäjä", 3),
    ("Sukellusvene", 3),
    ("Partioalus", 2),
];
