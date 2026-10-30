//! Pelien logiikka ilman ikkunaa, piirtoa tai syötettä. Kukin logiikka asuu oman
//! pelinsä kansiossa, muu osa kansiosta kuuluu ohjelmaan (`main.rs`).

pub mod arpa;
#[path = "pelit/laivanupotus/logiikka/mod.rs"]
pub mod laivanupotus;
#[path = "pelit/luvut/logiikka.rs"]
pub mod luvut;

pub use arpa::Arpa;
