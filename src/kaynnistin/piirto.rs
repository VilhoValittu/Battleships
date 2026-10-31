//! Käynnistimen näkymä: pelikortit ja ohjeet.

use super::{Kaynnissa, Kaynnistin, PELIT};
use crate::pelit::{asteroidit, laivanupotus, luvut, pasianssi, silmukka, tiilirikko};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{self, Kangas};

const KORTTI_X: i32 = 48;
const KORTTI_L: i32 = LEVEYS as i32 - 2 * KORTTI_X;
const KORTTI_Y: i32 = 360;
const KORTTI_K: i32 = 54;
const KORTTI_ASKEL: i32 = KORTTI_K + 3;
const KORTTI_SISENNYS: i32 = 18;
const KIELIKYTKIN: &str = "valikko.kielikytkin";
const KIELI_Y: i32 = 6;
const KIELI_VALI: i32 = 12;

/// Kytkin on äänikytkimen vasemmalla puolella, ja sen leveys seuraa tekstiä.
fn kielikytkimen_x() -> i32 {
    AANI_X - KIELI_VALI - napin_leveys(KIELIKYTKIN)
}

pub fn kielikytkimella(x: i32, y: i32) -> bool {
    let vasen = kielikytkimen_x();
    (vasen..vasen + napin_leveys(KIELIKYTKIN)).contains(&x)
        && (KIELI_Y..KIELI_Y + NAPPI_K).contains(&y)
}

#[cfg(test)]
pub fn kielikytkimen_keskipiste() -> (i32, i32) {
    (
        kielikytkimen_x() + napin_leveys(KIELIKYTKIN) / 2,
        KIELI_Y + NAPPI_K / 2,
    )
}

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

/// Piirtää esillä olevan näkymän ruudulle. Klassiset näkymät kulkevat välikuvan kautta.
pub fn piirra(kaynnistin: &Kaynnistin, ruutu: &mut [u32], kuva: &mut [u32]) {
    match &kaynnistin.peli {
        Some(Kaynnissa::Pasianssi(peli)) if peli.vaihe != pasianssi::peli::Vaihe::Alku => {
            return pasianssi::piirto::piirra(peli, ruutu);
        }
        Some(Kaynnissa::Pasianssi(peli)) => pasianssi::piirto::piirra_alku(peli, kuva),
        Some(Kaynnissa::Laivanupotus(peli)) => laivanupotus::piirto::piirra(peli, kuva),
        Some(Kaynnissa::Luvut(peli)) => luvut::piirto::piirra(peli, kuva),
        Some(Kaynnissa::Tiilirikko(peli)) => tiilirikko::piirto::piirra(peli, kuva),
        Some(Kaynnissa::Asteroidit(peli)) => asteroidit::piirto::piirra(peli, kuva),
        Some(Kaynnissa::Silmukka(peli)) => silmukka::piirto::piirra(peli, kuva),
        None => {
            let mut p = Piirtaja {
                k: Kangas::new(kuva),
                kello: kaynnistin.kello_ms,
            };
            p.valikko(kaynnistin.valittu);
            p.aanikytkin(kaynnistin.aanet_paalla);
            p.nappi(kielikytkimen_x(), KIELI_Y, KIELIKYTKIN, VALKOINEN, MUSTA);
        }
    }
    skaalaa_ruudulle(kuva, ruutu);
}

impl Piirtaja<'_> {
    fn valikko(&mut self, valittu: usize) {
        self.maisema(
            kaanna("valikko.banneri"),
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
            let nimi = format!("{} {}", i + 1, kaanna(kortti.nimi).to_uppercase());
            self.vektoriteksti(x, y + 4, &nimi, OTSIKKO, nimi_vari, true);
            self.teksti(x, y + 29, kaanna(kortti.kuvaus), MUSTA);
        }
        // Kuusi korttia vie muut rivit, joten ohjeesta jää vain tärkein ja hiiri löytyy itsestään.
        let ohje = kaanna("valikko.ohje");
        self.teksti(KORTTI_X, ALOITUS_Y, ohje, VALKOINEN);
        if self.vilkku() {
            let x = KORTTI_X + ohje.chars().count() as i32 * MERKKI;
            self.k.laatikko(x, ALOITUS_Y, MERKKI, MERKKI, REUNA);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;

    #[test]
    fn kortit_ja_ohje_mahtuvat_kuvaan() {
        const { assert!(KORTTI_Y + PELIT.len() as i32 * KORTTI_ASKEL <= ALOITUS_Y) };
        const { assert!(ALOITUS_Y + MERKKI < KORKEUS as i32) };
    }

    #[test]
    fn kielikytkin_loytyy_eika_mene_muiden_paalle() {
        kaikilla_kielilla(|| {
            let (x, y) = kielikytkimen_keskipiste();
            assert!(kielikytkimella(x, y));
            assert!(!kielikytkimella(AANI_X, y));
            assert!(!aanikytkimella(x, y));
            assert_eq!(kortti_pisteesta(x, y), None);
            assert!(kielikytkimen_x() > 0);
            for kortti in &PELIT {
                let leveys =
                    KORTTI_SISENNYS + kaanna(kortti.kuvaus).chars().count() as i32 * MERKKI;
                assert!(leveys <= KORTTI_L, "{}", kortti.kuvaus);
            }
            let ohje = kaanna("valikko.ohje").chars().count() as i32 + 1;
            assert!(KORTTI_X + ohje * MERKKI <= LEVEYS as i32);
        });
    }

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
