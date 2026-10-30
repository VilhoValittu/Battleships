use laivanupotus::{KOKO, Laivasto, Ruutu, SARAKKEET};

use super::fontti;
use super::peli::{Peli, Vaihe};

pub const LEVEYS: usize = 320;
pub const KORKEUS: usize = 256;

// Paletti: C64:n siniset ja Amiga Workbench 1.3:n käyttöliittymävärit.
const REUNA: u32 = 0x7C70DA;
const TAUSTA: u32 = 0x352879;
const VESI: u32 = 0x40318D;
const VESI_DITHER: u32 = 0x352879;
const RUUDUKKO: u32 = 0x241A5C;
const VALKOINEN: u32 = 0xFFFFFF;
const MUSTA: u32 = 0x000000;
const WB_SININEN: u32 = 0x0055AA;
const WB_ORANSSI: u32 = 0xFF8800;
const LAIVA: u32 = 0xADADAD;
const LAIVA_VALO: u32 = 0xE0E0E0;
const OSUMA: u32 = 0xB53A2E;
const HUTI: u32 = 0x6ABFC6;
const KELTAINEN: u32 = 0xEEEE77;

const SOLU: i32 = 11;
const LAUTA_Y: i32 = 32;
const VASTUSTAJA_X: i32 = 30;
const OMA_X: i32 = 190;
const TILASTO_Y: i32 = 147;
const TILASTO_SISENNYS: i32 = 16;
const TILASTO_PALKKI_X: i32 = 14 * fontti::LEVEYS;
const PALKIN_PALA: i32 = 6;
const PANEELI_Y: i32 = 201;
const PANEELI_RIVI_Y: i32 = PANEELI_Y + 3;
const RIVIN_KORKEUS: i32 = 8;
const PANEELI_RIVEJA: usize = 6;
const TEKSTI_X: i32 = 8;
const RIVILLE_MERKKEJA: usize = 38;
const NAPPI_Y: i32 = PANEELI_RIVI_Y + 5 * RIVIN_KORKEUS - 1;
const NAPPI_K: i32 = 10;
const AANI_PAALLA: &str = "Ääni: päällä";
const AANI_POIS: &str = "Ääni: pois";
// Mahtuu pidemmän tekstin mukaan, jottei kytkin muuta kokoaan.
const AANI_L: i32 = 12 * fontti::LEVEYS + 8;
const AANI_X: i32 = LEVEYS as i32 - 4 - AANI_L;
const AANI_Y: i32 = 2;
const AANI_K: i32 = 10;

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
    (Nappi::Uudestaan, "K = Uudestaan", NAPPI_K),
    (Nappi::Lopeta, "E = Lopeta", NAPPI_K + 13 * 8 + 16),
];

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
            (nx..nx + leveys).contains(&x) && (NAPPI_Y..NAPPI_Y + 10).contains(&y)
        })
        .map(|&(nappi, ..)| nappi)
}

pub fn aanikytkimella(x: i32, y: i32) -> bool {
    (AANI_X..AANI_X + AANI_L).contains(&x) && (AANI_Y..AANI_Y + AANI_K).contains(&y)
}

#[cfg(test)]
pub fn aanikytkimen_keskipiste() -> (i32, i32) {
    (AANI_X + AANI_L / 2, AANI_Y + AANI_K / 2)
}

fn napin_leveys(teksti: &str) -> i32 {
    teksti.chars().count() as i32 * fontti::LEVEYS + 8
}

#[cfg(test)]
pub fn ruudun_keskipiste(ruutu: Ruutu) -> (i32, i32) {
    (
        VASTUSTAJA_X + ruutu.sarake as i32 * SOLU + SOLU / 2,
        LAUTA_Y + ruutu.rivi as i32 * SOLU + SOLU / 2,
    )
}

#[cfg(test)]
pub fn napin_keskipiste(nappi: Nappi) -> (i32, i32) {
    let (_, teksti, x) = NAPIT.iter().find(|n| n.0 == nappi).copied().unwrap();
    (x + napin_leveys(teksti) / 2, NAPPI_Y + 5)
}

/// Rivittää tekstin sanojen välistä. Ei kopioi tekstiä, palauttaa vain viipaleet.
pub fn rivita(teksti: &str, leveys: usize) -> Vec<&str> {
    let mut rivit = Vec::new();
    let mut alku = 0;
    let mut viime_vali = None;
    let mut merkkeja = 0;
    for (i, merkki) in teksti.char_indices() {
        if merkkeja == leveys {
            let katko = viime_vali.unwrap_or(i);
            rivit.push(&teksti[alku..katko]);
            alku = if viime_vali.is_some() {
                katko + 1
            } else {
                katko
            };
            merkkeja = teksti[alku..i].chars().count();
            viime_vali = None;
        }
        if merkki == ' ' {
            viime_vali = Some(i);
        }
        merkkeja += 1;
    }
    if alku < teksti.len() {
        rivit.push(&teksti[alku..]);
    }
    rivit
}

pub fn piirra(peli: &Peli, vilkku: bool, kuva: &mut [u32]) {
    let mut p = Piirtaja(kuva);
    if peli.vaihe == Vaihe::Alku {
        p.alkuruutu(vilkku);
    } else {
        p.pelinakyma(peli, vilkku);
    }
    p.aanikytkin(peli.aanet_paalla);
}

struct Piirtaja<'a>(&'a mut [u32]);

impl Piirtaja<'_> {
    fn laatikko(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        let (a, b) = (x.max(0) as usize, (x + l).clamp(0, LEVEYS as i32) as usize);
        if a >= b {
            return;
        }
        for yy in y.max(0)..(y + k).min(KORKEUS as i32) {
            let rivi = yy as usize * LEVEYS;
            self.0[rivi + a..rivi + b].fill(vari);
        }
    }

    fn kehys(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        self.laatikko(x, y, l, 1, vari);
        self.laatikko(x, y + k - 1, l, 1, vari);
        self.laatikko(x, y, 1, k, vari);
        self.laatikko(x + l - 1, y, 1, k, vari);
    }

    fn piste(&mut self, x: i32, y: i32, vari: u32) {
        self.laatikko(x, y, 1, 1, vari);
    }

    fn teksti(&mut self, x: i32, y: i32, teksti: &str, vari: u32) {
        for (i, merkki) in teksti.chars().enumerate() {
            let vasen = x + i as i32 * fontti::LEVEYS;
            for (rivi, bitit) in fontti::kuvio(merkki).iter().enumerate() {
                for sarake in 0..8 {
                    if bitit >> sarake & 1 != 0 {
                        self.piste(vasen + sarake, y + rivi as i32, vari);
                    }
                }
            }
        }
    }

    fn alkuruutu(&mut self, vilkku: bool) {
        self.laatikko(0, 0, LEVEYS as i32, KORKEUS as i32, REUNA);
        self.laatikko(8, 8, LEVEYS as i32 - 16, KORKEUS as i32 - 16, TAUSTA);
        let mut y = 16;
        self.teksti(72, y, "**** LAIVANUPOTUS ****", REUNA);
        y += 2 * RIVIN_KORKEUS;
        for osa in [
            "Tervetuloa pelaamaan laivanupotusta.",
            "Ohjeet:",
            "Kummallakin on 10x10 ruudukossa 5 laivaa (pituudet 5, 4, 3, 3 ja 2).",
            "Ammutte vuorotellen. Voittaja on se, joka upottaa ensin vastustajan kaikki laivat.",
        ] {
            for rivi in rivita(osa, 36) {
                self.teksti(16, y, rivi, REUNA);
                y += RIVIN_KORKEUS;
            }
        }
        y += RIVIN_KORKEUS;
        for rivi in [
            "Hiiri: osoita ja klikkaa ruutua.",
            "Nuolet, A-J ja 1-0: valitse ruutu.",
            "Välilyönti tai Enter: ammu.",
            "M: äänet päälle tai pois.",
            "Esc: lopeta.",
        ] {
            self.teksti(16, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        y += RIVIN_KORKEUS;
        self.teksti(16, y, "Paina välilyöntiä tai klikkaa.", REUNA);
        y += RIVIN_KORKEUS;
        if vilkku {
            self.laatikko(16, y, 8, 8, REUNA);
        }
    }

    fn pelinakyma(&mut self, peli: &Peli, vilkku: bool) {
        self.laatikko(0, 0, LEVEYS as i32, KORKEUS as i32, TAUSTA);
        self.kehys(0, 0, LEVEYS as i32, KORKEUS as i32, REUNA);
        self.kehys(1, 1, LEVEYS as i32 - 2, KORKEUS as i32 - 2, REUNA);
        self.otsikkopalkki();

        self.teksti(VASTUSTAJA_X - 16, 15, "VASTUSTAJAN LAUTA", VALKOINEN);
        self.teksti(OMA_X - 16, 15, "OMA LAUTA", VALKOINEN);
        self.lauta(VASTUSTAJA_X, &peli.vastustaja, false);
        self.lauta(OMA_X, &peli.oma, true);
        if peli.vaihe == Vaihe::Ammunta {
            let x = VASTUSTAJA_X + peli.kursori.sarake as i32 * SOLU;
            let y = LAUTA_Y + peli.kursori.rivi as i32 * SOLU;
            let vari = if vilkku { KELTAINEN } else { WB_ORANSSI };
            self.kehys(x - 1, y - 1, SOLU + 3, SOLU + 3, vari);
            self.kehys(x, y, SOLU + 1, SOLU + 1, MUSTA);
        }
        self.tilasto(VASTUSTAJA_X - TILASTO_SISENNYS, &peli.vastustaja);
        self.tilasto(OMA_X - TILASTO_SISENNYS, &peli.oma);
        self.paneeli(peli);
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
        let mut y = TILASTO_Y + RIVIN_KORKEUS + 1;
        for laiva in laivasto.laivat() {
            let upotettu = laivasto.on_upotettu(laiva);
            let (teksti_vari, pala_vari) = if upotettu {
                (WB_ORANSSI, OSUMA)
            } else {
                (VALKOINEN, LAIVA)
            };
            self.teksti(x, y, laiva.nimi, teksti_vari);
            if upotettu {
                // yliviivaus, jottei tieto jää pelkän värin varaan
                let leveys = laiva.nimi.chars().count() as i32 * fontti::LEVEYS;
                self.laatikko(x, y + 3, leveys, 1, teksti_vari);
            }
            for i in 0..laiva.ruudut.maara() as i32 {
                let pala_x = x + TILASTO_PALKKI_X + i * PALKIN_PALA;
                self.laatikko(pala_x, y + 1, PALKIN_PALA - 1, 5, pala_vari);
            }
            y += RIVIN_KORKEUS;
        }
    }

    fn otsikkopalkki(&mut self) {
        self.laatikko(2, 2, LEVEYS as i32 - 4, 10, VALKOINEN);
        self.teksti(6, 3, "Laivanupotus", WB_SININEN);
        for y in [3, 5, 7, 9] {
            self.laatikko(110, y, LEVEYS as i32 - 116, 1, WB_SININEN);
        }
    }

    fn lauta(&mut self, x0: i32, laivasto: &Laivasto, nayta_laivat: bool) {
        for (i, kirjain) in SARAKKEET.chars().enumerate() {
            let x = x0 + i as i32 * SOLU + 2;
            self.teksti(x, LAUTA_Y - 8, kirjain.encode_utf8(&mut [0; 4]), REUNA);
        }
        for rivi in 0..KOKO {
            let y = LAUTA_Y + rivi as i32 * SOLU + 2;
            let tx = if rivi == 9 { x0 - 17 } else { x0 - 9 };
            self.teksti(tx, y, &(rivi + 1).to_string(), REUNA);
            for sarake in 0..KOKO {
                if let Some(ruutu) = Ruutu::new(rivi, sarake) {
                    self.solu(x0, ruutu, laivasto, nayta_laivat);
                }
            }
        }
        let koko = SOLU * KOKO as i32 + 1;
        self.kehys(x0, LAUTA_Y, koko, koko, RUUDUKKO);
    }

    fn solu(&mut self, x0: i32, ruutu: Ruutu, laivasto: &Laivasto, nayta_laivat: bool) {
        let x = x0 + ruutu.sarake as i32 * SOLU;
        let y = LAUTA_Y + ruutu.rivi as i32 * SOLU;
        self.laatikko(x, y, SOLU, SOLU, RUUDUKKO);
        self.laatikko(x + 1, y + 1, SOLU - 1, SOLU - 1, VESI);
        // pistekuvio jäljittelee 16 värin aikaa
        for d in (2..SOLU).step_by(4) {
            self.piste(x + d, y + d, VESI_DITHER);
        }
        let laiva = laivasto.on_laiva(ruutu);
        let ammuttu = laivasto.ammutut().sisaltaa(ruutu);
        if ammuttu && laiva {
            self.laatikko(x + 1, y + 1, SOLU - 1, SOLU - 1, OSUMA);
            for d in 2..SOLU - 1 {
                self.piste(x + d, y + d, VALKOINEN);
                self.piste(x + SOLU - d, y + d, VALKOINEN);
            }
        } else if ammuttu {
            self.laatikko(x + 4, y + 4, 3, 3, HUTI);
        } else if laiva && nayta_laivat {
            self.laatikko(x + 1, y + 1, SOLU - 1, SOLU - 1, LAIVA);
            self.laatikko(x + 1, y + 1, SOLU - 1, 1, LAIVA_VALO);
            self.laatikko(x + 1, y + 1, 1, SOLU - 1, LAIVA_VALO);
        }
    }

    fn paneeli(&mut self, peli: &Peli) {
        self.laatikko(4, PANEELI_Y, LEVEYS as i32 - 8, 52, WB_SININEN);
        self.kehys(4, PANEELI_Y, LEVEYS as i32 - 8, 52, VALKOINEN);

        let mut y = PANEELI_RIVI_Y;
        for viesti in &peli.viestit {
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
            Vaihe::Voitto | Vaihe::Tappio => {
                let teksti = if peli.vaihe == Vaihe::Voitto {
                    VOITTO
                } else {
                    TAPPIO
                };
                self.teksti(TEKSTI_X, alarivi - RIVIN_KORKEUS, teksti, WB_ORANSSI);
                self.napit();
            }
            Vaihe::Alku => {}
        }
    }

    fn aanikytkin(&mut self, paalla: bool) {
        let (tausta, teksti) = if paalla {
            (VALKOINEN, AANI_PAALLA)
        } else {
            (WB_ORANSSI, AANI_POIS)
        };
        self.laatikko(AANI_X, AANI_Y, AANI_L, AANI_K, tausta);
        self.kehys(AANI_X, AANI_Y, AANI_L, AANI_K, MUSTA);
        self.teksti(AANI_X + 4, AANI_Y + 1, teksti, MUSTA);
    }

    fn napit(&mut self) {
        for (_, teksti, x) in NAPIT {
            let leveys = napin_leveys(teksti);
            self.laatikko(x, NAPPI_Y, leveys, 10, VALKOINEN);
            self.laatikko(x + 1, NAPPI_Y + 9, leveys - 1, 1, MUSTA);
            self.laatikko(x + leveys - 1, NAPPI_Y + 1, 1, 9, MUSTA);
            self.teksti(x + 4, NAPPI_Y + 1, teksti, WB_SININEN);
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
    fn rivitys_katkaisee_sanavalista_ja_kasittelee_aakkoset() {
        let teksti = "Vastustaja ampui B10: OSUMA! Lentotukialus upposi!";
        let rivit = rivita(teksti, RIVILLE_MERKKEJA);
        assert_eq!(
            rivit,
            ["Vastustaja ampui B10: OSUMA!", "Lentotukialus upposi!"]
        );
        assert_eq!(rivita("ääääää", 4), ["ääää", "ää"]);
        assert!(rivita("", 4).is_empty());
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
}
