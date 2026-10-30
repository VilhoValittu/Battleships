use std::f32::consts::{PI, TAU};

use klassikot8000::laivanupotus::{KOKO, Laiva, Laivasto, Laukaustulos, Ruutu, SARAKKEET, Vaikeus};

use super::peli::{Peli, TULOS_VIIVE_MS, Tehoste, Vaihe};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{self, Kangas, kehapiste};

const SOLU: i32 = 33;
const LAUDAN_KOKO: i32 = SOLU * KOKO as i32;
const LAUTA_Y: i32 = 99;
const VASTUSTAJA_X: i32 = 90;
const OMA_X: i32 = 570;
const OTSIKKO_Y: i32 = 48;
const TILASTO_Y: i32 = 441;
const TILASTO_SISENNYS: i32 = 48;
const TILASTO_PALKKI_X: i32 = 14 * MERKKI;

const TEHOSTE_KESTO_MS: f32 = super::peli::TEHOSTE_KESTO_MS as f32;
const MERKINTA: (i32, i32, i32) = (12, 18, 18);

const OHJE: &str = "Ammu: klikkaa tai nuolet + välilyönti";
const AMMUIT_JO: &str = "Ammuit jo tähän!";
const VOITTO: &str = "****HIHIHIHIII VOITIT PELIN!****";
const TAPPIO: &str = "HÄVISIT PELIN!";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nappi {
    Uudestaan,
    Lopeta,
}

const NAPIT: [(Nappi, &str, i32); 2] = [
    (Nappi::Uudestaan, "K = Uudestaan", NAPPI_X),
    (Nappi::Lopeta, "E = Valikko", NAPPI_X + 13 * MERKKI + 48),
];

const TASO_OTSIKKO: &str = "Taso:";
const TASO_Y: i32 = ALOITUS_Y - NAPPI_K - 6;
const TASO_X: i32 = 48 + 6 * MERKKI;
const TASO_VALI: i32 = 12;
const TASOT: [(Vaikeus, &str, i32); 3] = [
    (Vaikeus::Helppo, "1 Helppo", TASO_X),
    (
        Vaikeus::Normaali,
        "2 Normaali",
        TASO_X + 8 * MERKKI + 24 + TASO_VALI,
    ),
    (
        Vaikeus::Vaikea,
        "3 Vaikea",
        TASO_X + 18 * MERKKI + 48 + 2 * TASO_VALI,
    ),
];

pub fn vaikeus_pisteesta(x: i32, y: i32) -> Option<Vaikeus> {
    TASOT
        .iter()
        .find(|&&(_, teksti, nx)| {
            (nx..nx + napin_leveys(teksti)).contains(&x) && (TASO_Y..TASO_Y + NAPPI_K).contains(&y)
        })
        .map(|&(vaikeus, ..)| vaikeus)
}

#[cfg(test)]
pub fn vaikeuden_keskipiste(vaikeus: Vaikeus) -> (i32, i32) {
    let (_, teksti, x) = TASOT.iter().find(|t| t.0 == vaikeus).copied().unwrap();
    (x + napin_leveys(teksti) / 2, TASO_Y + NAPPI_K / 2)
}

/// Vastustajan laudan ruutu annetussa pikselissä.
pub fn ruutu_pisteesta(x: i32, y: i32) -> Option<Ruutu> {
    let (dx, dy) = (x - VASTUSTAJA_X, y - LAUTA_Y);
    if dx < 0 || dy < 0 {
        return None;
    }
    Ruutu::new((dy / SOLU) as usize, (dx / SOLU) as usize)
}

pub fn nappi_pisteesta(x: i32, y: i32) -> Option<Nappi> {
    NAPIT
        .iter()
        .find(|&&(_, teksti, nx)| {
            let leveys = napin_leveys(teksti);
            (nx..nx + leveys).contains(&x) && (NAPPI_Y..NAPPI_Y + NAPPI_K).contains(&y)
        })
        .map(|&(nappi, ..)| nappi)
}

#[cfg(test)]
pub fn ruudun_keskipiste(ruutu: Ruutu) -> (i32, i32) {
    solun_keskipiste(VASTUSTAJA_X, ruutu)
}

fn solun_keskipiste(x0: i32, ruutu: Ruutu) -> (i32, i32) {
    (
        x0 + ruutu.sarake as i32 * SOLU + SOLU / 2,
        LAUTA_Y + ruutu.rivi as i32 * SOLU + SOLU / 2,
    )
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + NAPPI_K / 2)
}

pub fn piirra(peli: &Peli, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas(kuva),
        kello: peli.kello_ms,
    };
    if peli.vaihe == Vaihe::Alku {
        p.alkuruutu(peli.vaikeus);
    } else {
        p.pelinakyma(peli);
    }
    p.aanikytkin(peli.aanet_paalla);
}

impl Piirtaja<'_> {
    fn alkuruutu(&mut self, vaikeus: Vaikeus) {
        self.maisema(
            "**** LAIVANUPOTUS ****",
            &vektori::taistelulaiva(RUNKO, KANSI, TORNI),
        );

        let mut y = OHJE_Y;
        for osa in [
            "Tervetuloa pelaamaan laivanupotusta.",
            "Ohjeet:",
            "Kummallakin on 10x10 ruudukossa 5 laivaa (pituudet 5, 4, 3, 3 ja 2).",
            "Ammutte vuorotellen. Voittaja on se, joka upottaa ensin vastustajan kaikki laivat.",
        ] {
            for rivi in rivita(osa, 36) {
                self.teksti(48, y, rivi, REUNA);
                y += RIVIN_KORKEUS;
            }
        }
        y += RIVIN_KORKEUS / 2;
        for rivi in [
            "Hiiri: osoita ja klikkaa ruutua.",
            "Nuolet, A-J ja 1-0: valitse ruutu.",
            "Välilyönti tai Enter: ammu.",
            "M: äänet päälle tai pois.",
            "Esc: takaisin valikkoon.",
        ] {
            self.teksti(48, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        self.tasovalinta(vaikeus);
        y = ALOITUS_Y;
        self.teksti(48, y, "Paina välilyöntiä tai klikkaa.", REUNA);
        if self.vilkku() {
            self.k.laatikko(48 + 30 * MERKKI, y, MERKKI, MERKKI, REUNA);
        }
    }

    fn tasovalinta(&mut self, valittu: Vaikeus) {
        self.teksti(48, TASO_Y + 3, TASO_OTSIKKO, KELTAINEN);
        for (vaikeus, teksti, x) in TASOT {
            if vaikeus == valittu {
                self.nappi(x, TASO_Y, teksti, WB_ORANSSI, MUSTA);
            } else {
                self.nappi(x, TASO_Y, teksti, VALKOINEN, WB_SININEN);
            }
        }
    }

    fn pelinakyma(&mut self, peli: &Peli) {
        self.liuku(0, KORKEUS as i32, TAUSTA_TUMMA, TAUSTA);
        let (l, k) = (LEVEYS as i32, KORKEUS as i32);
        self.k.kehys(0, 0, l, k, REUNA);
        self.k.kehys(1, 1, l - 2, k - 2, REUNA);
        self.otsikkopalkki("Laivanupotus", peli.vaikeus.nimi());

        self.vektoriteksti(
            VASTUSTAJA_X,
            OTSIKKO_Y,
            "VASTUSTAJAN LAUTA",
            OTSIKKO,
            VALKOINEN,
            false,
        );
        self.vektoriteksti(OMA_X, OTSIKKO_Y, "OMA LAUTA", OTSIKKO, VALKOINEN, false);
        self.lauta(VASTUSTAJA_X, &peli.vastustaja, false, peli);
        self.lauta(OMA_X, &peli.oma, true, peli);
        if peli.vaihe == Vaihe::Ammunta {
            self.tahtain(peli.kursori);
        }
        self.tilasto(VASTUSTAJA_X - TILASTO_SISENNYS, &peli.vastustaja);
        self.tilasto(OMA_X - TILASTO_SISENNYS, &peli.oma);
        self.paneeli(peli);
        for tehoste in &peli.tehosteet {
            self.tehoste(tehoste);
        }
    }

    /// Laudan alle: montako laivaa on upotettu ja mitkä. Osumia ei kerrota,
    /// koska vastustajan laivoista pelaajan kuuluu tietää vain uponneet.
    fn tilasto(&mut self, x: i32, laivasto: &Laivasto) {
        let otsikko = format!(
            "Upotettu {}/{}",
            laivasto.upotettuja(),
            laivasto.laivat().len()
        );
        self.teksti(x, TILASTO_Y, &otsikko, KELTAINEN);
        let mut y = TILASTO_Y + RIVIN_KORKEUS + 3;
        for laiva in laivasto.laivat() {
            let upotettu = laivasto.on_upotettu(laiva);
            let (teksti_vari, pala_vari) = if upotettu {
                (WB_ORANSSI, OSUMA)
            } else {
                (VALKOINEN, KANSI)
            };
            self.teksti(x, y, laiva.nimi, teksti_vari);
            if upotettu {
                // yliviivaus, jottei tieto jää pelkän värin varaan
                let leveys = laiva.nimi.chars().count() as i32 * MERKKI;
                self.k.laatikko(x, y + 9, leveys, 3, teksti_vari);
            }
            for i in 0..laiva.ruudut.maara() as i32 {
                let pala_x = x + TILASTO_PALKKI_X + i * PALKIN_PALA;
                self.k
                    .laatikko(pala_x, y + 3, PALKIN_PALA - 3, 15, pala_vari);
            }
            y += RIVIN_KORKEUS;
        }
    }

    fn lauta(&mut self, x0: i32, laivasto: &Laivasto, omalle: bool, peli: &Peli) {
        for (i, kirjain) in SARAKKEET.chars().enumerate() {
            let x = x0 + i as i32 * SOLU + 10;
            let mut puskuri = [0; 4];
            let merkki = kirjain.encode_utf8(&mut puskuri);
            self.vektoriteksti(x, LAUTA_Y - 22, merkki, MERKINTA, REUNA, false);
        }
        for rivi in 0..KOKO {
            let y = LAUTA_Y + rivi as i32 * SOLU + 7;
            let tx = if rivi == 9 { x0 - 39 } else { x0 - 21 };
            let numero = (rivi + 1).to_string();
            self.vektoriteksti(tx, y, &numero, MERKINTA, REUNA, false);
        }

        self.k.laatikko(x0, LAUTA_Y, LAUDAN_KOKO, LAUDAN_KOKO, VESI);
        self.aallot(x0);
        for i in 0..=KOKO as i32 {
            let viiva = x0 + i * SOLU;
            let rivi = LAUTA_Y + i * SOLU;
            self.k.laatikko(
                viiva.min(x0 + LAUDAN_KOKO),
                LAUTA_Y,
                1,
                LAUDAN_KOKO + 1,
                RUUDUKKO,
            );
            self.k.laatikko(x0, rivi, LAUDAN_KOKO + 1, 1, RUUDUKKO);
        }
        if omalle {
            for laiva in laivasto.laivat() {
                self.laiva(x0, laiva);
            }
        }
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                if let Some(ruutu) = Ruutu::new(rivi, sarake) {
                    let ammuttu = laivasto.ammutut().sisaltaa(ruutu);
                    if ammuttu && !peli.odottaa_osumaa(omalle, ruutu) {
                        self.laukaus(x0, ruutu, laivasto.on_laiva(ruutu));
                    }
                }
            }
        }
        self.k
            .kehys(x0 - 1, LAUTA_Y - 1, LAUDAN_KOKO + 3, LAUDAN_KOKO + 3, REUNA);
    }

    /// Hitaat aaltoviivat elävöittävät vettä peittämättä merkintöjä.
    fn aallot(&mut self, x0: i32) {
        const VALI: i32 = 33;
        for rivi in 0..KOKO as i32 {
            let y = LAUTA_Y + rivi * SOLU + SOLU / 2;
            let vaihe = self.kello as f32 * 0.0025 + rivi as f32 * 1.4;
            let mut pisteet = [(0, 0); 11];
            for (i, piste) in pisteet.iter_mut().enumerate() {
                let x = i as i32 * VALI;
                let korkeus = (vaihe + i as f32 * 0.9).sin() * 3.5;
                *piste = (x0 + x, y + korkeus as i32);
            }
            self.k.murtoviiva(&pisteet, AALTO);
        }
    }

    /// Laiva ylhäältä: suippo keula, kansirakenne ja tykkitornit joka ruudussa.
    fn laiva(&mut self, x0: i32, laiva: &Laiva) {
        let mut rajat: Option<(i32, i32, i32)> = None;
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                let Some(ruutu) = Ruutu::new(rivi, sarake) else {
                    continue;
                };
                if laiva.ruudut.sisaltaa(ruutu) {
                    let (r, s) = (rivi as i32, sarake as i32);
                    rajat = Some(match rajat {
                        Some((r0, s0, r1)) => (r0.min(r), s0.min(s), r1.max(r)),
                        None => (r, s, r),
                    });
                }
            }
        }
        let Some((r0, s0, r1)) = rajat else {
            return;
        };
        let vaaka = r0 == r1;
        let pituus = laiva.ruudut.maara() as i32 * SOLU;
        let (vasen, yla) = (x0 + s0 * SOLU, LAUTA_Y + r0 * SOLU);
        // u kulkee perästä keulaan, v laidasta laitaan
        let p = |u: i32, v: i32| {
            if vaaka {
                (vasen + u, yla + SOLU / 2 + v)
            } else {
                (vasen + SOLU / 2 + v, yla + u)
            }
        };
        let runko = [
            p(3, -8),
            p(pituus - 18, -11),
            p(pituus - 3, 0),
            p(pituus - 18, 11),
            p(3, 8),
        ];
        self.k.monikulmio(&runko, RUNKO);
        for (i, &a) in runko.iter().enumerate() {
            self.k.viiva(a, runko[(i + 1) % runko.len()], LAIVA_REUNA);
        }
        let kansi = pituus / 3;
        let rakenne = [
            p(kansi, -4),
            p(pituus - kansi - 3, -4),
            p(pituus - kansi - 3, 4),
            p(kansi, 4),
        ];
        self.k.monikulmio(&rakenne, TORNI);
        for osa in 0..laiva.ruudut.maara() as i32 {
            let keski = p(osa * SOLU + SOLU / 2, 0);
            self.k.ympyra(keski, 4.0, LAIVA_REUNA);
        }
    }

    fn laukaus(&mut self, x0: i32, ruutu: Ruutu, osui: bool) {
        let keski = solun_keskipiste(x0, ruutu);
        if osui {
            let x = x0 + ruutu.sarake as i32 * SOLU + 1;
            let y = LAUTA_Y + ruutu.rivi as i32 * SOLU + 1;
            self.k.laatikko(x, y, SOLU - 1, SOLU - 1, OSUMA_TUMMA);
            // liekki väpättää, mutta eri ruudut eri tahtiin
            let tahti = self.kello / 110 + ruutu.rivi as u32 * 7 + ruutu.sarake as u32 * 3;
            for i in 0..8 {
                let kulma = i as f32 / 8.0 * TAU + 0.2;
                let sade = 8.0 + ((tahti + i) % 3) as f32 * 3.0;
                let kaari = kehapiste(keski, sade, kulma);
                let vari = if i % 2 == 0 { KELTAINEN } else { WB_ORANSSI };
                self.k.viiva(keski, kaari, vari);
            }
            self.k.laatikko(keski.0 - 1, keski.1 - 1, 3, 3, VALKOINEN);
        } else {
            self.k.ympyra(keski, 9.0, HUTI);
            self.k.laatikko(keski.0 - 1, keski.1 - 1, 3, 3, HUTI);
        }
    }

    /// Tähtäin: kulmat ja keskelle osoittavat viivat sykkivät.
    fn tahtain(&mut self, kursori: Ruutu) {
        let keski = solun_keskipiste(VASTUSTAJA_X, kursori);
        let vari = if self.vilkku() { KELTAINEN } else { WB_ORANSSI };
        let (sade, kulma) = (SOLU / 2 + 1, 8);
        let (cx, cy) = keski;
        for (sx, sy) in [(-1, -1), (1, -1), (1, 1), (-1, 1)] {
            let nurkka = (cx + sx * sade, cy + sy * sade);
            self.k
                .paksu_viiva(nurkka, (nurkka.0 - sx * kulma, nurkka.1), vari);
            self.k
                .paksu_viiva(nurkka, (nurkka.0, nurkka.1 - sy * kulma), vari);
        }
        let sisa = 5 + (self.kello / 150 % 3) as i32 * 2;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            self.k.viiva(
                (cx + dx * sisa, cy + dy * sisa),
                (cx + dx * (sisa + 6), cy + dy * (sisa + 6)),
                vari,
            );
        }
    }

    fn tehoste(&mut self, tehoste: &Tehoste) {
        if self.kello < tehoste.laukaus_ms {
            return;
        }
        let (x0, lahto_x) = if tehoste.omalle {
            (OMA_X, VASTUSTAJA_X)
        } else {
            (VASTUSTAJA_X, OMA_X)
        };
        let kohde = solun_keskipiste(x0, tehoste.ruutu);
        let kulunut = self.kello - tehoste.laukaus_ms;
        if kulunut < TULOS_VIIVE_MS {
            let lahto = (lahto_x + LAUDAN_KOKO / 2, LAUTA_Y + LAUDAN_KOKO / 2);
            self.ammus(lahto, kohde, kulunut as f32 / TULOS_VIIVE_MS as f32);
            return;
        }
        let ika = (kulunut - TULOS_VIIVE_MS) as f32 / TEHOSTE_KESTO_MS;
        match tehoste.tulos {
            Laukaustulos::Ohi => self.roiske(kohde, ika),
            Laukaustulos::Osuma => self.rajahdys(kohde, ika, 12, 42.0),
            Laukaustulos::Upposi(_) => self.rajahdys(kohde, ika, 24, 105.0),
        }
    }

    /// Ammus lentää kaarena toiselta laudalta; häntä näyttää suunnan.
    fn ammus(&mut self, lahto: (i32, i32), kohde: (i32, i32), t: f32) {
        let paikka = |t: f32| {
            let x = lahto.0 as f32 + (kohde.0 - lahto.0) as f32 * t;
            let y = lahto.1 as f32 + (kohde.1 - lahto.1) as f32 * t;
            (x as i32, (y - (t * PI).sin() * 90.0) as i32)
        };
        let karki = paikka(t);
        self.k
            .paksu_viiva(paikka((t - 0.18).max(0.0)), karki, KELTAINEN);
        self.k.laatikko(karki.0 - 1, karki.1 - 1, 3, 3, VALKOINEN);
    }

    fn roiske(&mut self, keski: (i32, i32), ika: f32) {
        let vari = sekoita(VALKOINEN, VESI, ika);
        for i in 0..3 {
            let sade = (ika * 39.0 - i as f32 * 9.0).max(0.0);
            if sade > 0.0 {
                self.k.ympyra(keski, sade, vari);
            }
        }
    }

    fn paneeli(&mut self, peli: &Peli) {
        let leveys = LEVEYS as i32 - 12;
        self.k.laatikko(6, PANEELI_Y, leveys, PANEELI_K, WB_SININEN);
        self.k.kehys(6, PANEELI_Y, leveys, PANEELI_K, VALKOINEN);

        let vastaus_kesken = peli.vastaus_kesken();
        let mut y = PANEELI_RIVI_Y;
        for (i, viesti) in peli.viestit.iter().enumerate() {
            if i == 1 && vastaus_kesken {
                continue;
            }
            for rivi in rivita(viesti, RIVILLE_MERKKEJA) {
                self.teksti(TEKSTI_X, y, rivi, VALKOINEN);
                y += RIVIN_KORKEUS;
            }
        }

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        match peli.vaihe {
            Vaihe::Ammunta if peli.ammuttu_jo => {
                self.teksti(TEKSTI_X, alarivi, AMMUIT_JO, WB_ORANSSI);
            }
            Vaihe::Ammunta => self.teksti(TEKSTI_X, alarivi, OHJE, HUTI),
            Vaihe::Voitto | Vaihe::Tappio if !vastaus_kesken => {
                let teksti = if peli.vaihe == Vaihe::Voitto {
                    VOITTO
                } else {
                    TAPPIO
                };
                let y = alarivi - RIVIN_KORKEUS - 2;
                self.vektoriteksti(TEKSTI_X, y, teksti, TULOS, KELTAINEN, true);
                self.napit();
            }
            Vaihe::Voitto | Vaihe::Tappio | Vaihe::Alku => {}
        }
    }

    fn napit(&mut self) {
        for (_, teksti, x) in NAPIT {
            self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jokainen_ruutu_loytyy_keskipisteestaan() {
        for rivi in 0..KOKO {
            for sarake in 0..KOKO {
                let ruutu = Ruutu::new(rivi, sarake).unwrap();
                let (x, y) = ruudun_keskipiste(ruutu);
                assert_eq!(ruutu_pisteesta(x, y), Some(ruutu));
            }
        }
    }

    #[test]
    fn oma_lauta_ja_reunat_eivat_ole_kohteita() {
        assert_eq!(ruutu_pisteesta(OMA_X + 5, LAUTA_Y + 5), None);
        assert_eq!(ruutu_pisteesta(VASTUSTAJA_X - 1, LAUTA_Y), None);
        assert_eq!(ruutu_pisteesta(VASTUSTAJA_X, LAUTA_Y + SOLU * 10), None);
    }

    #[test]
    fn pisin_kierros_mahtuu_paneeliin() {
        // kaksi viestiä, lopputulos ja napit
        let a = "Sinä ammuit B10: OSUMA! Lentotukialus upposi!";
        let b = "Vastustaja ampui B10: OSUMA! Lentotukialus upposi!";
        let rivit = rivita(a, RIVILLE_MERKKEJA).len() + rivita(b, RIVILLE_MERKKEJA).len();
        assert!(rivit + 2 <= PANEELI_RIVEJA);
        assert!(OHJE.chars().count() <= RIVILLE_MERKKEJA);
    }

    #[test]
    fn kaikki_vaiheet_piirtyvat_kaatumatta_tehosteiden_aikana() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut peli = Peli::new(3);
        piirra(&peli, &mut kuva);
        peli.vaihe = Vaihe::Ammunta;
        for i in 0..KOKO * KOKO {
            peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
            for _ in 0..4 {
                peli.edista(100);
                piirra(&peli, &mut kuva);
            }
        }
    }
}
