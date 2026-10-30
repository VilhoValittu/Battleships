use crate::{Arpa, KOKO, Ruudut, Ruutu};

/// Tasaisen satunnainen ruutu niistä, joihin ei ole ammuttu. `None`, jos lauta on täynnä.
pub fn valitse(ammutut: Ruudut, arpa: &mut Arpa) -> Option<Ruutu> {
    let vapaita = KOKO * KOKO - ammutut.maara();
    if vapaita == 0 {
        return None;
    }
    let indeksi = arpa.alle(vapaita);
    (0..KOKO * KOKO)
        .filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
        .filter(|&ruutu| !ammutut.sisaltaa(ruutu))
        .nth(indeksi)
}
