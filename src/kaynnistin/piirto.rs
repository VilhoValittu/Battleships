//! Käynnistimen näkymä: pelikortit ja ohjeet.

use super::{Kaynnissa, Kaynnistin, PELIT};
use crate::pelit::{asteroidit, laivanupotus, luvut, pasianssi, silmukka, tiilirikko};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{self, Kangas};

// Käynnistin piirtää suoraan ruudulle, joten mitat ovat ruudun pikseleitä.
const RUUTU_L: i32 = RUUDUN_LEVEYS as i32;

const KORTTI_X: i32 = 48;
const KORTTI_L: i32 = 704;
const KORTTI_Y: i32 = 172;
const KORTTI_K: i32 = 120;
const KORTTI_ASKEL: i32 = KORTTI_K + 10;
const KORTIN_NUMERO_X: i32 = 60;
const KORTIN_NIMI: (i32, i32, i32) = (24, 36, 32);
const KORTIN_NIMEN_PAKSUUS: i32 = 3;
const KORTIN_VARJO: i32 = 6;

const OTSIKKO_KOKO: (i32, i32, i32) = (60, 88, 78);
const OTSIKKO_Y: i32 = 48;
const OTSIKON_PAKSUUS: i32 = 4;
const OTSIKON_VARJO: i32 = 5;

// Meri ei vyöry: muuten koko alaosa vaihtuisi joka ruudussa, vaikka katsoja seuraa mallia.
const MAISEMA: Maisema = Maisema {
    horisontti: 520,
    ala_y: ALAPALKKI_Y,
    katoamis_x: MALLI_X,
    vyory: false,
};
const MALLI_X: i32 = 1336;
const MALLI_Y: i32 = 460;
const MALLIN_KOKO: f32 = 76.0;
const MALLIN_KEINUNTA: f32 = 6.0;

const PANEELI_X: i32 = 800;
const PANEELI_L: i32 = 1072;
const PANEELI_YLA: i32 = KORTTI_Y + 5 * KORTTI_ASKEL + KORTTI_K - PANEELI_KORKEUS;
const PANEELI_KORKEUS: i32 = 180;
const PANEELI_SISENNYS: i32 = 24;

const TEKSTIN_KERROIN: i32 = 4;
const SUURI_MERKKI: i32 = TEKSTIN_KERROIN * 8;
const KUVAUS_RIVI: i32 = 40;
const KUVAUS_MERKKEJA: usize = ((PANEELI_L - 2 * PANEELI_SISENNYS) / SUURI_MERKKI) as usize;

const ALAPALKKI_Y: i32 = 976;
const OHJE_X: i32 = 48;
const OHJE_RIVI_Y: i32 = 1012;

const KIELIKYTKIN: &str = "valikko.kielikytkin";
const KIELI_Y: i32 = 6;
const KIELI_VALI: i32 = 12;
// Yhteinen äänikytkin piirtyy kankaan oikeaan laitaan, jossa on nyt koko ruudun leveys.
const AANI_SIIRTO: i32 = (RUUDUN_LEVEYS - LEVEYS) as i32;

/// Kytkin on äänikytkimen vasemmalla puolella, ja sen leveys seuraa tekstiä.
fn kielikytkimen_x() -> i32 {
    AANI_X + AANI_SIIRTO - KIELI_VALI - napin_leveys(KIELIKYTKIN)
}

pub fn aanikytkimella(x: i32, y: i32) -> bool {
    crate::yhteinen::piirto::aanikytkimella(x - AANI_SIIRTO, y)
}

#[cfg(test)]
pub fn aanikytkimen_keskipiste() -> (i32, i32) {
    let (x, y) = crate::yhteinen::piirto::aanikytkimen_keskipiste();
    (x + AANI_SIIRTO, y)
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

/// Piirtää esillä olevan näkymän ruudulle. Pelien klassiset näkymät kulkevat välikuvan kautta.
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
                k: Kangas::kokoinen(ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS),
                kello: kaynnistin.kello_ms,
            };
            p.valikko(kaynnistin.valittu);
            p.aanikytkin(kaynnistin.aanet_paalla);
            p.nappi(kielikytkimen_x(), KIELI_Y, KIELIKYTKIN, VALKOINEN, MUSTA);
            return;
        }
    }
    skaalaa_ruudulle(kuva, ruutu);
}

impl Piirtaja<'_> {
    fn valikko(&mut self, valittu: usize) {
        self.maisematausta(MAISEMA);
        self.valikon_malli();
        self.valikon_otsikko(kaanna("valikko.banneri"));
        for i in 0..PELIT.len() {
            self.valikkokortti(i, i == valittu);
        }
        self.kuvauspaneeli(PELIT[valittu].kuvaus);
        self.ohjepalkki(kaanna("valikko.ohje"));
    }

    fn valikon_malli(&mut self) {
        let malli = vektori::taistelulaiva(RUNKO, KANSI, TORNI);
        let keinunta = (self.kello as f32 * 0.002).sin() * MALLIN_KEINUNTA;
        self.k.malli(
            &malli,
            self.kello as f32 * LAIVAN_KIERTO,
            (MALLI_X, MALLI_Y + keinunta as i32),
            MALLIN_KOKO,
            LAIVA_REUNA,
        );
    }

    fn valikon_otsikko(&mut self, teksti: &str) {
        let x = (RUUTU_L - teksti.chars().count() as i32 * OTSIKKO_KOKO.2) / 2;
        let varjo = OTSIKON_VARJO;
        let (koko, paksuus) = (OTSIKKO_KOKO, OTSIKON_PAKSUUS);
        self.levea_vektoriteksti(x + varjo, OTSIKKO_Y + varjo, teksti, koko, paksuus, TAUSTA);
        self.levea_vektoriteksti(x, OTSIKKO_Y, teksti, koko, paksuus, KELTAINEN);
    }

    fn valikkokortti(&mut self, i: usize, valittu: bool) {
        let y = KORTTI_Y + i as i32 * KORTTI_ASKEL;
        let keski_y = y + KORTTI_K / 2;
        let (numero_vari, nimi_vari) = if valittu {
            self.k.laatikko(
                KORTTI_X + KORTIN_VARJO,
                y + KORTIN_VARJO,
                KORTTI_L,
                KORTTI_K,
                MUSTA,
            );
            self.k.laatikko(KORTTI_X, y, KORTTI_L, KORTTI_K, WB_ORANSSI);
            self.k.monikulmio(
                &[
                    (KORTTI_X + 20, keski_y - 18),
                    (KORTTI_X + 44, keski_y),
                    (KORTTI_X + 20, keski_y + 18),
                ],
                MUSTA,
            );
            (MUSTA, MUSTA)
        } else {
            self.k
                .laatikko(KORTTI_X, y, KORTTI_L, KORTTI_K, TAUSTA_TUMMA);
            self.k.kehys(KORTTI_X, y, KORTTI_L, KORTTI_K, REUNA);
            (KELTAINEN, VALKOINEN)
        };
        let (koko, paksuus) = (KORTIN_NIMI, KORTIN_NIMEN_PAKSUUS);
        let teksti_y = keski_y - koko.1 / 2;
        let x = KORTTI_X + KORTIN_NUMERO_X;
        let numero = (i + 1).to_string();
        self.levea_vektoriteksti(x, teksti_y, &numero, koko, paksuus, numero_vari);
        let nimi = kaanna(PELIT[i].nimi).to_uppercase();
        self.levea_vektoriteksti(x + 2 * koko.2, teksti_y, &nimi, koko, paksuus, nimi_vari);
    }

    fn kuvauspaneeli(&mut self, kuvaus: &str) {
        self.k.laatikko(
            PANEELI_X,
            PANEELI_YLA,
            PANEELI_L,
            PANEELI_KORKEUS,
            TAUSTA_TUMMA,
        );
        self.k
            .kehys(PANEELI_X, PANEELI_YLA, PANEELI_L, PANEELI_KORKEUS, REUNA);
        let x = PANEELI_X + PANEELI_SISENNYS;
        let mut y = PANEELI_YLA + PANEELI_SISENNYS;
        for rivi in rivita(kaanna(kuvaus), KUVAUS_MERKKEJA) {
            self.teksti_koossa(x, y, rivi, TEKSTIN_KERROIN, VALKOINEN);
            y += KUVAUS_RIVI;
        }
    }

    fn ohjepalkki(&mut self, ohje: &str) {
        let korkeus = RUUDUN_KORKEUS as i32 - ALAPALKKI_Y;
        self.k.laatikko(0, ALAPALKKI_Y, RUUTU_L, korkeus, TAUSTA);
        self.k.laatikko(0, ALAPALKKI_Y, RUUTU_L, 3, REUNA);
        self.teksti_koossa(OHJE_X, OHJE_RIVI_Y, ohje, TEKSTIN_KERROIN, VALKOINEN);
        if self.vilkku() {
            let x = OHJE_X + ohje.chars().count() as i32 * SUURI_MERKKI;
            self.k
                .laatikko(x, OHJE_RIVI_Y, SUURI_MERKKI, SUURI_MERKKI, REUNA);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;

    #[test]
    fn kortit_ja_paneelit_mahtuvat_kuvaan() {
        const { assert!(KORTTI_Y + PELIT.len() as i32 * KORTTI_ASKEL <= ALAPALKKI_Y) };
        const { assert!(KORTTI_X + KORTTI_L + KORTIN_VARJO < PANEELI_X) };
        const { assert!(PANEELI_X + PANEELI_L < RUUTU_L) };
        const { assert!(OHJE_RIVI_Y + SUURI_MERKKI < RUUDUN_KORKEUS as i32) };
    }

    #[test]
    fn kielikytkin_loytyy_eika_mene_muiden_paalle() {
        kaikilla_kielilla(|| {
            let (x, y) = kielikytkimen_keskipiste();
            assert!(kielikytkimella(x, y));
            let (aani_x, aani_y) = aanikytkimen_keskipiste();
            assert!(aanikytkimella(aani_x, aani_y));
            assert!(!kielikytkimella(aani_x, aani_y));
            assert!(!aanikytkimella(x, y));
            assert_eq!(kortti_pisteesta(x, y), None);
            assert!(kielikytkimen_x() > 0);
        });
    }

    #[test]
    fn tekstit_mahtuvat_paikoilleen_kummallakin_kielella() {
        kaikilla_kielilla(|| {
            for kortti in &PELIT {
                let nimi = kaanna(kortti.nimi).chars().count() as i32;
                let oikea = KORTIN_NUMERO_X + (2 + nimi) * KORTIN_NIMI.2;
                assert!(oikea <= KORTTI_L, "{}", kortti.nimi);
                let rivit = rivita(kaanna(kortti.kuvaus), KUVAUS_MERKKEJA).len() as i32;
                let korkeus = 2 * PANEELI_SISENNYS + rivit * KUVAUS_RIVI;
                assert!(korkeus <= PANEELI_KORKEUS, "{}", kortti.kuvaus);
            }
            let ohje = kaanna("valikko.ohje").chars().count() as i32 + 1;
            assert!(OHJE_X + ohje * SUURI_MERKKI <= RUUTU_L);
            let otsikko = kaanna("valikko.banneri").chars().count() as i32;
            assert!(otsikko * OTSIKKO_KOKO.2 <= RUUTU_L);
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
