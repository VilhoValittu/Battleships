//! Pelien logiikka ilman ikkunaa, piirtoa tai syötettä. Kukin logiikka asuu oman
//! pelinsä kansiossa, muu osa kansiosta kuuluu ohjelmaan (`main.rs`).

pub mod arpa;
#[path = "pelit/asteroidit/logiikka.rs"]
pub mod asteroidit;
#[path = "pelit/laivanupotus/logiikka/mod.rs"]
pub mod laivanupotus;
#[path = "pelit/luvut/logiikka.rs"]
pub mod luvut;
#[path = "pelit/miinaharava/logiikka.rs"]
pub mod miinaharava;
#[path = "pelit/palikka/logiikka.rs"]
pub mod palikka;
#[path = "pelit/pasianssi/logiikka.rs"]
pub mod pasianssi;
#[path = "pelit/silmukka/logiikka.rs"]
pub mod silmukka;
#[path = "pelit/tiilirikko/logiikka.rs"]
pub mod tiilirikko;

pub use arpa::Arpa;
