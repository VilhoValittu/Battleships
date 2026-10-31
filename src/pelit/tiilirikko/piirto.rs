//! Tiilirikon näkymät.

use klassikot8000::tiilirikko::{
    self, KENTTIA, MAILA_K, MAILA_Y, PALLON_SADE, RIVEJA, SARAKKEITA, TIILET_Y, TIILI_K, TIILI_L,
    Tila,
};

use super::peli::{KIPINA_MS, Nappi, Tiilirikko, Vaihe};
use crate::yhteinen::kieli::kaanna;
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio};

pub const KENTTA_X: i32 = 30;
const KENTTA_Y: i32 = 48;
const KENTTA_L: i32 = tiilirikko::LEVEYS as i32;
const KENTTA_K: i32 = tiilirikko::KORKEUS as i32;
const SARAKE_X: i32 = KENTTA_X + KENTTA_L + 30;
const TILASTON_ASKEL: i32 = 87;
const PARHAAT_Y: i32 = KENTTA_Y + 4 * TILASTON_ASKEL + 66;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;
// Kapeaan sarakkeeseen mahtuu sijoitus, kahdeksan merkin nimi ja viisinumeroiset pisteet.
const PARHAAT_KOKO: (i32, i32, i32) = (10, 15, 14);

const RIVIN_VARIT: [u32; RIVEJA] = [
    OSUMA, WB_ORANSSI, KELTAINEN, 0x70B050, HUTI, 0x4FA8E0, REUNA, 0x8B3FC0,
];

const OHJE: &str = "tiilirikko.ohje";
const LAUKAISU: &str = "tiilirikko.laukaisu";
const TAUKO: &str = "yhteinen.tauko";
const TAUON_OHJE: &str = "yhteinen.tauon_ohje";
const VOITTO: &str = "tiilirikko.voitto";
const TAPPIO: &str = "yhteinen.peli_paattyi";

const NAPIT: [(Nappi, &str, i32); 2] = [
    (Nappi::Uudestaan, "yhteinen.uudestaan", NAPPI_X),
    (
        Nappi::Valikko,
        "yhteinen.valikkoon",
        NAPPI_X + 13 * MERKKI + 48,
    ),
];

const ALKU_OSAT: [&str; 7] = [
    "tiilirikko.tervetuloa",
    "yhteinen.ohjeet",
    "tiilirikko.ohje_1",
    "tiilirikko.ohje_2",
    "tiilirikko.ohje_3",
    "tiilirikko.ohje_4",
    "tiilirikko.ohje_5",
];
const ALKU_RIVIT: [&str; 4] = [
    "tiilirikko.nappain_hiiri",
    "tiilirikko.nappain_laukaisu",
    "yhteinen.tauko_aanet",
    "yhteinen.takaisin",
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
pub fn tiilimuuri() -> Vec<Tahko> {
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
        p.tiili_alkuruutu(peli);
    } else {
        p.tiili_nakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

impl Piirtaja<'_> {
    fn tiili_alkuruutu(&mut self, peli: &Tiilirikko) {
        self.maisema(kaanna("tiilirikko.banneri"), &tiilimuuri());
        let otsikko = listan_otsikko(peli.netti_parhaat.is_some());
        self.parhaat_alkuruudulla(otsikko, peli.nakyva_lista());
        self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
        self.aloituskehote(ALOITUS);
    }

    fn tiili_nakyma(&mut self, peli: &Tiilirikko) {
        self.liuku(0, KORKEUS as i32, TAUSTA_TUMMA, TAUSTA);
        let (l, k) = (LEVEYS as i32, KORKEUS as i32);
        self.k.kehys(0, 0, l, k, REUNA);
        self.k.kehys(1, 1, l - 2, k - 2, REUNA);
        self.otsikkopalkki(&kaanna("peli.tiilirikko").to_uppercase(), "");

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
                let pallotiili = peli.lauta.pallotiilet[rivi][sarake];
                let vari = if pallotiili { VALKOINEN } else { vari };
                self.k.laatikko(x + 1, y + 1, l - 2, k - 2, vari);
                self.k
                    .laatikko(x + 1, y + 1, l - 2, 3, sekoita(vari, VALKOINEN, 0.5));
                self.k
                    .laatikko(x + 1, y + k - 4, l - 2, 3, sekoita(vari, MUSTA, 0.4));
                // Pallotiilen keskellä on pallon kokoinen merkki.
                if pallotiili {
                    let sivu = 2 * PALLON_SADE as i32;
                    self.k
                        .laatikko(x + (l - sivu) / 2, y + (k - sivu) / 2, sivu, sivu, MUSTA);
                }
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
        let (l, k) = (lauta.maila_l() as i32, MAILA_K as i32);
        let x = KENTTA_X + (lauta.maila_x - lauta.maila_l() / 2.0) as i32;
        let y = KENTTA_Y + MAILA_Y as i32;
        self.k.laatikko(x, y, l, k, KANSI);
        self.k.laatikko(x, y, l, 3, VALKOINEN);
        self.k.laatikko(x, y, 12, k, WB_ORANSSI);
        self.k.laatikko(x + l - 12, y, 12, k, WB_ORANSSI);

        let sivu = 2 * PALLON_SADE as i32;
        for pallo in &lauta.pallot {
            let px = KENTTA_X + (pallo.paikka.0 - PALLON_SADE) as i32;
            let py = KENTTA_Y + (pallo.paikka.1 - PALLON_SADE) as i32;
            // Pallon alapuoli voi pudota kentän alle, mutta paneelin päälle se ei piirry.
            let korkeus = sivu.min(KENTTA_Y + KENTTA_K - py);
            if korkeus > 0 {
                self.k.laatikko(px, py, sivu, korkeus, VALKOINEN);
            }
        }
    }

    fn tiili_tilasto(&mut self, peli: &Tiilirikko) {
        let lauta = &peli.lauta;
        let kentta = format!("{}/{KENTTIA}", lauta.kentta + 1);
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), lauta.pisteet.to_string()),
            (kaanna("yhteinen.paras"), peli.paras.to_string()),
            (kaanna("tiilirikko.kentta"), kentta),
        ]
        .into_iter()
        .enumerate()
        {
            let y = KENTTA_Y + i as i32 * TILASTON_ASKEL;
            self.vektoriteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            self.vektoriteksti(SARAKE_X, y + 30, &arvo, OTSIKKO, VALKOINEN, true);
        }

        let y = KENTTA_Y + 3 * TILASTON_ASKEL;
        self.vektoriteksti(
            SARAKE_X,
            y,
            kaanna("tiilirikko.pallot"),
            OTSIKKO,
            KELTAINEN,
            false,
        );
        for i in 0..lauta.elamat as i32 {
            self.k
                .laatikko(SARAKE_X + i * 27, y + 33, 18, 18, VALKOINEN);
        }

        let y = KENTTA_Y + 4 * TILASTON_ASKEL;
        let bonus = lauta.aikabonus().to_string();
        self.vektoriteksti(
            SARAKE_X,
            y,
            kaanna("tiilirikko.aikabonus"),
            TULOS,
            KELTAINEN,
            false,
        );
        self.vektoriteksti(SARAKE_X, y + 30, &bonus, OTSIKKO, VALKOINEN, true);

        self.parhaat_lista(
            SARAKE_X,
            PARHAAT_Y,
            PARHAAT_L,
            PARHAAT_KOKO,
            listan_otsikko(peli.netti_parhaat.is_some()),
            peli.nakyva_lista(),
        );
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
                self.teksti(TEKSTI_X, alarivi, kaanna(LAUKAISU), KELTAINEN);
            }
            Vaihe::Pelissa => self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI),
            Vaihe::Tauko => {
                self.vektoriteksti(TEKSTI_X, otsikko_y, kaanna(TAUKO), TULOS, KELTAINEN, true);
                self.teksti(TEKSTI_X, alarivi, kaanna(TAUON_OHJE), HUTI);
            }
            Vaihe::Voitto | Vaihe::Tappio => {
                let otsikko = if peli.vaihe == Vaihe::Voitto {
                    VOITTO
                } else {
                    TAPPIO
                };
                self.vektoriteksti(TEKSTI_X, otsikko_y, kaanna(otsikko), TULOS, KELTAINEN, true);
                for (_, teksti, x) in NAPIT {
                    self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
                }
            }
            Vaihe::Nimi => self.nimikysely(peli.lauta.pisteet, &peli.nimi),
            Vaihe::Alku => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::{kaikilla_kielilla, muotoile};

    #[test]
    fn napit_loytyvat_eivatka_mene_paallekkain() {
        for (nappi, ..) in NAPIT {
            let (x, y) = napin_keskipiste(nappi);
            assert_eq!(nappi_pisteesta(x, y), Some(nappi));
        }
        assert_eq!(nappi_pisteesta(0, 0), None);
        kaikilla_kielilla(|| assert!(NAPIT[0].2 + napin_leveys(NAPIT[0].1) < NAPIT[1].2));
    }

    #[test]
    fn tekstit_ja_kentta_mahtuvat_paikoilleen() {
        kaikilla_kielilla(|| {
            for teksti in [
                kaanna(OHJE).to_string(),
                kaanna(LAUKAISU).to_string(),
                kaanna(TAUON_OHJE).to_string(),
                kaanna("tiilirikko.uusi_pallo").to_string(),
                kaanna("tiilirikko.toinen_meni").to_string(),
                muotoile("tiilirikko.pallo_meni", &[&3]),
                muotoile("tiilirikko.kentta_lapi", &[&5000]),
                muotoile("tiilirikko.loppubonus", &[&5000, &99999]),
            ] {
                assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
            }
            let voitto = kaanna(VOITTO).chars().count() as i32;
            assert!(voitto * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
            for avain in ["tiilirikko.kentta", "tiilirikko.pallot"] {
                let leveys = kaanna(avain).chars().count() as i32 * OTSIKKO.2;
                assert!(SARAKE_X + leveys < LEVEYS as i32, "{avain}");
            }
            let bonus = kaanna("tiilirikko.aikabonus").chars().count() as i32 * TULOS.2;
            assert!(SARAKE_X + bonus < LEVEYS as i32);
        });
        alkuohje_mahtuu(&ALKU_OSAT, &ALKU_RIVIT, ALOITUS_Y);
        const { assert!((2 + 8 + 1 + 5) * PARHAAT_KOKO.2 <= PARHAAT_L) };
        const { assert!(PARHAAT_Y + 30 + 3 * 26 < PANEELI_Y) };
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
        peli.vaihe = Vaihe::Nimi;
        piirra(&peli, &mut kuva);
    }
}
