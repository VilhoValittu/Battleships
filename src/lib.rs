//! Pelien logiikka ilman ikkunaa, piirtoa tai syötettä. Kukin logiikka asuu oman
//! pelinsä kansiossa, muu osa kansiosta kuuluu ohjelmaan (`main.rs`).

pub mod arpa;
#[path = "pelit/laivanupotus/logiikka/mod.rs"]
pub mod laivanupotus;
#[path = "pelit/luvut/logiikka.rs"]
pub mod luvut;
#[path = "pelit/pasianssi/logiikka.rs"]
pub mod pasianssi;

pub use arpa::Arpa;
