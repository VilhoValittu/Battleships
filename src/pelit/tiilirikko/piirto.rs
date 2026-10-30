//! Tiilirikon näkymät.

use klassikot8000::tiilirikko::{
    self, KENTTIA, MAILA_K, MAILA_L, MAILA_Y, PALLON_SADE, RIVEJA, SARAKKEITA, TIILET_Y, TIILI_K,
    TIILI_L, Tila,
};

use super::peli::{KIPINA_MS, Nappi, Tiilirikko, Vaihe};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio};

pub const KENTTA_X: i32 = 30;
const KENTTA_Y: i32 = 48;
const KENTTA_L: i32 = tiilirikko::LEVEYS as i32;
const KENTTA_K: i32 = tiilirikko::KORKEUS as i32;
const SARAKE_X: i32 = KENTTA_X + KENTTA_L + 30;
const TILASTON_ASKEL: i32 = 87;

const RIVIN_VARIT: [u32; RIVEJA] = [
    OSUMA, WB_ORANSSI, KELTAINEN, 0x70B050, HUTI, 0x4FA8E0, REUNA, 0x8B3FC0,
];

const OHJE: &str = "Ohjaa: hiiri tai nuolet. P = tauko.";
const LAUKAISU: &str = "Laukaise: välilyönti tai klikkaus.";
const TAUKO: &str = "TAUKO";
const TAUON_OHJE: &str = "Jatka: P, välilyönti tai klikkaus.";
const VOITTO: &str = "****KAIKKI KENTÄT LÄPI!****";
const TAPPIO: &str = "PELI PÄÄTTYI!";

const NAPIT: [(Nappi, &str, i32); 2] = [
    (Nappi::Uudestaan, "K = Uudestaan", NAPPI_X),
    (Nappi::Valikko, "E = Valikko", NAPPI_X + 13 * MERKKI + 48),
];

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    NAPIT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x)
                && (NAPPI_Y..NAPPI_Y + NAPPI_K).contains(&y)
        })
        .map(|&(nappi, ..)| nappi)
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

fn tiilen_paikka(rivi: usize, sarake: usize) -> (i32, i32) {
    (
        KENTTA_X + sarake as i32 * TIILI_L as i32,
        KENTTA_Y + TIILET_Y as i32 + rivi as i32 * TIILI_K as i32,
    )
}

/// Alkuruudun malli: tiilimuuri ja sen edessä maila.
fn tiilimuuri() -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    for (rivi, &vari) in RIVIN_VARIT[..3].iter().enumerate() {
        for sarake in 0..4 {
            let x = -4.4 + sarake as f32 * 2.2;
            let y = 0.9 + rivi as f32 * 0.75;
            kuutio([x, x + 2.0], [y, y + 0.65], [-0.6, 0.6], vari, &mut tahkot);
        }
    }
    kuutio([-1.6, 1.6], [0.0, 0.3], [-0.6, 0.6], TORNI, &mut tahkot);
    tahkot
}

pub fn piirra(peli: &Tiilirikko, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.tiili_alkuruutu();
    } else {
        p.tiili_nakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

impl Piirtaja<'_> {
    fn tiili_alkuruutu(&mut self) {
        self.maisema("**** TIILIRIKKO ****", &tiilimuuri());
        let mut y = OHJE_Y;
        for osa in [
            "Tervetuloa pelaamaan Tiilirikkoa.",
            "Ohjeet:",
            "Pidä pallo kentällä mailalla ja riko kaikki tiilet.",
            "Mailan laita kääntää palloa jyrkemmin, joten voit tähdätä.",
            "Kenttiä on viisi, ja palloja on kolme.",
        ] {
            for rivi in rivita(osa, 36) {
                self.teksti(48, y, rivi, REUNA);
                y += RIVIN_KORKEUS;
            }
        }
        y += RIVIN_KORKEUS / 2;
        for rivi in [
            "Hiiri tai nuolet: liikuta mailaa.",
            "Välilyönti tai klikkaus: laukaise.",
            "P: tauko. M: äänet.",
            "Esc: takaisin valikkoon.",
        ] {
            self.teksti(48, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        self.teksti(48, ALOITUS_Y, "Paina välilyöntiä tai klikkaa.", REUNA);
        if self.vilkku() {
            self.k
                .laatikko(48 + 30 * MERKKI, ALOITUS_Y, MERKKI, MERKKI, REUNA);
        }
    }

    fn tiili_nakyma(&mut self, peli: &Tiilirikko) {
        self.liuku(0, KORKEUS as i32, TAUSTA_TUMMA, TAUSTA);
        let (l, k) = (LEVEYS as i32, KORKEUS as i32);
        self.k.kehys(0, 0, l, k, REUNA);
        self.k.kehys(1, 1, l - 2, k - 2, REUNA);
        self.otsikkopalkki("TIILIRIKKO", "");

        self.k
            .laatikko(KENTTA_X, KENTTA_Y, KENTTA_L, KENTTA_K, VESI);
        self.k.kehys(
            KENTTA_X - 1,
            KENTTA_Y - 1,
            KENTTA_L + 2,
            KENTTA_K + 2,
            REUNA,
        );
        self.tiilet(peli);
        self.maila_ja_pallo(peli);
        self.tiili_tilasto(peli);
        self.tiili_paneeli(peli);
    }

    fn tiilet(&mut self, peli: &Tiilirikko) {
        let (l, k) = (TIILI_L as i32, TIILI_K as i32);
        for (rivi, &vari) in RIVIN_VARIT.iter().enumerate() {
            for sarake in 0..SARAKKEITA {
                let kesto = peli.lauta.tiilet[rivi][sarake];
                if kesto == 0 {
                    continue;
                }
                let (x, y) = tiilen_paikka(rivi, sarake);
                self.k.laatikko(x + 1, y + 1, l - 2, k - 2, vari);
                self.k
                    .laatikko(x + 1, y + 1, l - 2, 3, sekoita(vari, VALKOINEN, 0.5));
                self.k
                    .laatikko(x + 1, y + k - 4, l - 2, 3, sekoita(vari, MUSTA, 0.4));
                // Kestävän tiilen keskellä näkyy yksi niitti jokaista lisäosumaa kohti.
                for niitti in 1..kesto as i32 {
                    let nx = x + l / 2 + (2 * niitti - kesto as i32) * 9 - 4;
                    self.k.laatikko(nx, y + k / 2 - 4, 8, 8, MUSTA);
                    self.k.laatikko(nx + 2, y + k / 2 - 2, 4, 4, VALKOINEN);
                }
            }
        }
        for &(rivi, sarake, alku) in &peli.kipinat {
            let (x, y) = tiilen_paikka(rivi, sarake);
            let ika = self.kello.saturating_sub(alku) as f32 / KIPINA_MS as f32;
            self.rajahdys((x + l / 2, y + k / 2), ika, 8, 45.0);
        }
    }

    fn maila_ja_pallo(&mut self, peli: &Tiilirikko) {
        let lauta = &peli.lauta;
        let (l, k) = (MAILA_L as i32, MAILA_K as i32);
        let x = KENTTA_X + (lauta.maila_x - MAILA_L / 2.0) as i32;
        let y = KENTTA_Y + MAILA_Y as i32;
        self.k.laatikko(x, y, l, k, KANSI);
        self.k.laatikko(x, y, l, 3, VALKOINEN);
        self.k.laatikko(x, y, 12, k, WB_ORANSSI);
        self.k.laatikko(x + l - 12, y, 12, k, WB_ORANSSI);

        if lauta.tila == Tila::Ohi {
            return;
        }
        let sivu = 2 * PALLON_SADE as i32;
        let px = KENTTA_X + (lauta.pallo.0 - PALLON_SADE) as i32;
        let py = KENTTA_Y + (lauta.pallo.1 - PALLON_SADE) as i32;
        // Pallon alapuoli voi pudota kentän alle, mutta paneelin päälle se ei piirry.
        let korkeus = sivu.min(KENTTA_Y + KENTTA_K - py);
        if korkeus > 0 {
            self.k.laatikko(px, py, sivu, korkeus, VALKOINEN);
        }
    }

    fn tiili_tilasto(&mut self, peli: &Tiilirikko) {
        let lauta = &peli.lauta;
        let kentta = format!("{}/{KENTTIA}", lauta.kentta + 1);
        for (i, (otsikko, arvo)) in [
            ("PISTEET", lauta.pisteet.to_string()),
            ("PARAS", peli.paras.to_string()),
            ("KENTTÄ", kentta),
        ]
        .into_iter()
        .enumerate()
        {
            let y = KENTTA_Y + i as i32 * TILASTON_ASKEL;
            self.vektoriteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            self.vektoriteksti(SARAKE_X, y + 30, &arvo, OTSIKKO, VALKOINEN, true);
        }

        let y = KENTTA_Y + 3 * TILASTON_ASKEL;
        self.vektoriteksti(SARAKE_X, y, "PALLOT", OTSIKKO, KELTAINEN, false);
        for i in 0..lauta.elamat as i32 {
            self.k
                .laatikko(SARAKE_X + i * 27, y + 33, 18, 18, VALKOINEN);
        }
    }

    fn tiili_paneeli(&mut self, peli: &Tiilirikko) {
        let leveys = LEVEYS as i32 - 12;
        self.k.laatikko(6, PANEELI_Y, leveys, PANEELI_K, WB_SININEN);
        self.k.kehys(6, PANEELI_Y, leveys, PANEELI_K, VALKOINEN);
        self.teksti(TEKSTI_X, PANEELI_RIVI_Y, &peli.viesti, VALKOINEN);

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        match peli.vaihe {
            Vaihe::Pelissa if peli.lauta.tila == Tila::Kiinni => {
                self.teksti(TEKSTI_X, alarivi, LAUKAISU, KELTAINEN);
            }
            Vaihe::Pelissa => self.teksti(TEKSTI_X, alarivi, OHJE, HUTI),
            Vaihe::Tauko => {
                self.vektoriteksti(TEKSTI_X, otsikko_y, TAUKO, TULOS, KELTAINEN, true);
                self.teksti(TEKSTI_X, alarivi, TAUON_OHJE, HUTI);
            }
            Vaihe::Voitto | Vaihe::Tappio => {
                let otsikko = if peli.vaihe == Vaihe::Voitto {
                    VOITTO
                } else {
                    TAPPIO
                };
                self.vektoriteksti(TEKSTI_X, otsikko_y, otsikko, TULOS, KELTAINEN, true);
                for (_, teksti, x) in NAPIT {
                    self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
                }
            }
            Vaihe::Alku => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn napit_loytyvat_eivatka_mene_paallekkain() {
        for (nappi, ..) in NAPIT {
            let (x, y) = napin_keskipiste(nappi);
            assert_eq!(nappi_pisteesta(x, y), Some(nappi));
        }
        assert_eq!(nappi_pisteesta(0, 0), None);
        assert!(NAPIT[0].2 + napin_leveys(NAPIT[0].1) < NAPIT[1].2);
    }

    #[test]
    fn tekstit_ja_kentta_mahtuvat_paikoilleen() {
        for teksti in [OHJE, LAUKAISU, TAUON_OHJE, "Kenttä läpi! Seuraavana 5/5."] {
            assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
        }
        assert!(VOITTO.chars().count() as i32 * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
        const { assert!(KENTTA_Y + KENTTA_K < PANEELI_Y) };
        assert!(SARAKE_X + "PISTEET".len() as i32 * OTSIKKO.2 < LEVEYS as i32);
    }

    #[test]
    fn koko_peli_piirtyy_kaatumatta() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut peli = Tiilirikko::new(3);
        piirra(&peli, &mut kuva);
        peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
        peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
        // Mailaa ei liikuteta, joten pallot putoavat kentän ali ja peli päättyy.
        for _ in 0..3000 {
            peli.edista(16);
            piirra(&peli, &mut kuva);
            if peli.lauta.tila == Tila::Kiinni {
                peli.kasittele(crate::yhteinen::syote::Syote::Ammu);
            }
        }
        peli.vaihe = Vaihe::Tauko;
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Voitto;
        piirra(&peli, &mut kuva);
    }
}
