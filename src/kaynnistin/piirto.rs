//! Käynnistimen näkymä: pelikortit ja ohjeet.

use super::{Kaynnissa, Kaynnistin, PELIT};
use crate::pelit::{laivanupotus, luvut};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{self, Kangas};

const KORTTI_X: i32 = 48;
const KORTTI_L: i32 = LEVEYS as i32 - 2 * KORTTI_X;
const KORTTI_Y: i32 = 372;
const KORTTI_K: i32 = 84;
const KORTTI_ASKEL: i32 = KORTTI_K + 12;
const KORTTI_SISENNYS: i32 = 18;

pub fn kortti_pisteesta(x: i32, y: i32) -> Option<usize> {
    if !(KORTTI_X..KORTTI_X + KORTTI_L).contains(&x) {
        return None;
    }
    let i = (y - KORTTI_Y).div_euclid(KORTTI_ASKEL);
    let sisalla = (y - KORTTI_Y).rem_euclid(KORTTI_ASKEL) < KORTTI_K;
    (sisalla && (0..PELIT.len() as i32).contains(&i)).then_some(i as usize)
}

#[cfg(test)]
pub fn kortin_keskipiste(i: usize) -> (i32, i32) {
    (
        KORTTI_X + KORTTI_L / 2,
        KORTTI_Y + i as i32 * KORTTI_ASKEL + KORTTI_K / 2,
    )
}

pub fn piirra(kaynnistin: &Kaynnistin, kuva: &mut [u32]) {
    match &kaynnistin.peli {
        Some(Kaynnissa::Laivanupotus(peli)) => return laivanupotus::piirto::piirra(peli, kuva),
        Some(Kaynnissa::Luvut(peli)) => return luvut::piirto::piirra(peli, kuva),
        None => {}
    }
    let mut p = Piirtaja {
        k: Kangas(kuva),
        kello: kaynnistin.kello_ms,
    };
    p.valikko(kaynnistin.valittu);
    p.aanikytkin(kaynnistin.aanet_paalla);
}

impl Piirtaja<'_> {
    fn valikko(&mut self, valittu: usize) {
        self.maisema(
            "**** PELIKONE ****",
            &vektori::taistelulaiva(RUNKO, KANSI, TORNI),
        );
        for (i, kortti) in PELIT.iter().enumerate() {
            let y = KORTTI_Y + i as i32 * KORTTI_ASKEL;
            let (tausta, nimi_vari) = if i == valittu {
                (WB_ORANSSI, MUSTA)
            } else {
                (VALKOINEN, WB_SININEN)
            };
            self.k.laatikko(KORTTI_X, y, KORTTI_L, KORTTI_K, tausta);
            self.k
                .laatikko(KORTTI_X + 3, y + KORTTI_K, KORTTI_L, 3, MUSTA);
            self.k
                .laatikko(KORTTI_X + KORTTI_L, y + 3, 3, KORTTI_K, MUSTA);
            let x = KORTTI_X + KORTTI_SISENNYS;
            let nimi = format!("{} {}", i + 1, kortti.nimi);
            self.vektoriteksti(x, y + 15, &nimi, OTSIKKO, nimi_vari, true);
            self.teksti(x, y + 48, kortti.kuvaus, MUSTA);
        }
        let mut y = KORTTI_Y + PELIT.len() as i32 * KORTTI_ASKEL + 12;
        for rivi in [
            "Nuolet, numero tai hiiri: valitse.",
            "Välilyönti tai klikkaus: pelaa.",
            "M: äänet päälle tai pois.",
            "Esc: lopeta.",
        ] {
            self.teksti(KORTTI_X, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        if self.vilkku() {
            self.k.laatikko(KORTTI_X, y, MERKKI, MERKKI, REUNA);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jokainen_kortti_loytyy_ja_valit_eivat_ole_kohteita() {
        for i in 0..PELIT.len() {
            let (x, y) = kortin_keskipiste(i);
            assert_eq!(kortti_pisteesta(x, y), Some(i));
        }
        assert_eq!(kortti_pisteesta(KORTTI_X, KORTTI_Y + KORTTI_K), None);
        assert_eq!(kortti_pisteesta(KORTTI_X, KORTTI_Y - 1), None);
        assert_eq!(kortti_pisteesta(KORTTI_X - 1, KORTTI_Y), None);
        let alin = KORTTI_Y + PELIT.len() as i32 * KORTTI_ASKEL;
        assert_eq!(kortti_pisteesta(KORTTI_X, alin), None);
    }
}
