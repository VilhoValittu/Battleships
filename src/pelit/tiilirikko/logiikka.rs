//! Tiilirikon säännöt ilman ikkunaa, piirtoa tai syötettä. Mitat ovat kentän pikseleitä,
//! ja aika etenee kiinteinä askelina, joten sama siemen ja ohjaus toistavat saman pelin.

use crate::Arpa;

pub const LEVEYS: f32 = 660.0;
pub const KORKEUS: f32 = 540.0;
pub const SARAKKEITA: usize = 11;
pub const RIVEJA: usize = 8;
pub const TIILI_L: f32 = LEVEYS / SARAKKEITA as f32;
pub const TIILI_K: f32 = 24.0;
pub const TIILET_Y: f32 = 48.0;
pub const MAILA_L: f32 = 108.0;
pub const MAILA_K: f32 = 15.0;
pub const MAILA_Y: f32 = KORKEUS - 39.0;
pub const PALLON_SADE: f32 = 7.0;
pub const ASKEL_MS: u32 = 5;
pub const ELAMIA: u32 = 3;

const ASKEL_S: f32 = ASKEL_MS as f32 / 1000.0;
const MAILAN_VAUHTI: f32 = 780.0;
const PERUSVAUHTI: f32 = 330.0;
const VAUHTI_KENTTAA_KOHTI: f32 = 30.0;
const VAUHTI_TIILTA_KOHTI: f32 = 4.0;
// Askel on pallon sädettä lyhyempi tälläkin vauhdilla, joten pallo ei hyppää tiilen yli.
const SUURIN_VAUHTI: f32 = 660.0;
const SUURIN_KULMA: f32 = 1.05;
const PISTEET_OSUMASTA: u32 = 10;

/// Numero kertoo, montako osumaa tiili kestää.
const KENTAT: [&[&str]; 5] = [
    &[
        "11111111111",
        "11111111111",
        "11111111111",
        "11111111111",
        "11111111111",
    ],
    &[
        "...........",
        ".....1.....",
        "....111....",
        "...11111...",
        "..1111111..",
        ".111111111.",
        "11111111111",
    ],
    &[
        "22222222222",
        "...........",
        "11111111111",
        "...........",
        "22222222222",
        "11111111111",
    ],
    &[
        "1.1.1.1.1.1",
        ".2.2.2.2.2.",
        "1.1.1.1.1.1",
        ".2.2.2.2.2.",
        "1.1.1.1.1.1",
        ".2.2.2.2.2.",
    ],
    &[
        "22222222222",
        "2.........2",
        "2.3.3.3.3.2",
        "2.........2",
        "2.3.3.3.3.2",
        "2.........2",
        "1.1.1.1.1.1",
    ],
];

pub const KENTTIA: usize = KENTAT.len();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tila {
    /// Pallo odottaa mailan päällä laukaisua.
    Kiinni,
    Lennossa,
    Voitto,
    Ohi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tapahtuma {
    Seina,
    Maila,
    Tiili {
        rivi: usize,
        sarake: usize,
        rikki: bool,
    },
    Menetys,
    KenttaLapi,
    Voitto,
    Ohi,
}

pub struct Tiilipeli {
    pub tiilet: [[u8; SARAKKEITA]; RIVEJA],
    pub maila_x: f32,
    pub pallo: (f32, f32),
    pub nopeus: (f32, f32),
    pub tila: Tila,
    pub elamat: u32,
    pub pisteet: u32,
    /// Nollasta alkava kentän numero.
    pub kentta: usize,
    vauhti: f32,
}

fn kentan_tiilet(kentta: usize) -> [[u8; SARAKKEITA]; RIVEJA] {
    let mut tiilet = [[0; SARAKKEITA]; RIVEJA];
    for (rivi, kuvio) in tiilet.iter_mut().zip(KENTAT[kentta]) {
        for (tiili, merkki) in rivi.iter_mut().zip(kuvio.chars()) {
            *tiili = merkki.to_digit(10).unwrap_or(0) as u8;
        }
    }
    tiilet
}

impl Tiilipeli {
    /// Peli alkaa annetusta kentästä, joka rajataan olemassa oleviin.
    pub fn new(kentta: usize) -> Self {
        let kentta = kentta.min(KENTTIA - 1);
        let mut peli = Self {
            tiilet: kentan_tiilet(kentta),
            maila_x: LEVEYS / 2.0,
            pallo: (0.0, 0.0),
            nopeus: (0.0, 0.0),
            tila: Tila::Kiinni,
            elamat: ELAMIA,
            pisteet: 0,
            kentta,
            vauhti: 0.0,
        };
        peli.kiinnita_pallo();
        peli
    }

    pub fn tiilia_jaljella(&self) -> usize {
        self.tiilet.iter().flatten().filter(|&&t| t > 0).count()
    }

    fn kiinnita_pallo(&mut self) {
        self.tila = Tila::Kiinni;
        self.nopeus = (0.0, 0.0);
        self.pallo = (self.maila_x, MAILA_Y - PALLON_SADE);
    }

    /// Maila pysyy kokonaan kentällä, ja kiinni oleva pallo kulkee sen mukana.
    pub fn aseta_maila(&mut self, x: f32) {
        self.maila_x = x.clamp(MAILA_L / 2.0, LEVEYS - MAILA_L / 2.0);
        if self.tila == Tila::Kiinni {
            self.pallo.0 = self.maila_x;
        }
    }

    pub fn laukaise(&mut self, arpa: &mut Arpa) {
        if self.tila != Tila::Kiinni {
            return;
        }
        self.tila = Tila::Lennossa;
        self.vauhti = PERUSVAUHTI + self.kentta as f32 * VAUHTI_KENTTAA_KOHTI;
        // Arvottu kulma estää lähdön suoraan ylös, jolloin pallo toistaisi samaa rataa.
        let kulma = (10 + arpa.alle(26)) as f32 * std::f32::consts::PI / 180.0;
        let suunta = if arpa.alle(2) == 0 { -1.0 } else { 1.0 };
        self.nopeus = (
            suunta * self.vauhti * kulma.sin(),
            -self.vauhti * kulma.cos(),
        );
    }

    /// Etenee yhden `ASKEL_MS`:n verran. `ohjaus` on -1 vasemmalle, 1 oikealle, 0 paikallaan.
    pub fn askel(&mut self, ohjaus: i8) -> Vec<Tapahtuma> {
        let mut tapahtumat = Vec::new();
        if matches!(self.tila, Tila::Voitto | Tila::Ohi) {
            return tapahtumat;
        }
        self.aseta_maila(self.maila_x + ohjaus as f32 * MAILAN_VAUHTI * ASKEL_S);
        if self.tila != Tila::Lennossa {
            return tapahtumat;
        }

        // Akselit erikseen: törmäyksen suunta on aina selvä, eikä pallo jää tiilen sisään.
        self.pallo.0 += self.nopeus.0 * ASKEL_S;
        let sivuseina = if self.pallo.0 < PALLON_SADE {
            Some(PALLON_SADE)
        } else if self.pallo.0 > LEVEYS - PALLON_SADE {
            Some(LEVEYS - PALLON_SADE)
        } else {
            None
        };
        if let Some(x) = sivuseina {
            self.pallo.0 = x;
            self.nopeus.0 = -self.nopeus.0;
            tapahtumat.push(Tapahtuma::Seina);
        } else if let Some(paikka) = self.osuva_tiili() {
            self.pallo.0 -= self.nopeus.0 * ASKEL_S;
            self.nopeus.0 = -self.nopeus.0;
            self.osu_tiileen(paikka, &mut tapahtumat);
            if self.tila != Tila::Lennossa {
                return tapahtumat;
            }
        }

        self.pallo.1 += self.nopeus.1 * ASKEL_S;
        if self.pallo.1 < PALLON_SADE {
            self.pallo.1 = PALLON_SADE;
            self.nopeus.1 = -self.nopeus.1;
            tapahtumat.push(Tapahtuma::Seina);
        } else if let Some(paikka) = self.osuva_tiili() {
            self.pallo.1 -= self.nopeus.1 * ASKEL_S;
            self.nopeus.1 = -self.nopeus.1;
            self.osu_tiileen(paikka, &mut tapahtumat);
            if self.tila != Tila::Lennossa {
                return tapahtumat;
            }
        }

        if self.osuu_mailaan() {
            self.pallo.1 = MAILA_Y - PALLON_SADE;
            // Mailan laita kääntää pallon jyrkemmin, joten pelaaja voi tähdätä.
            let osuma = (self.pallo.0 - self.maila_x) / (MAILA_L / 2.0 + PALLON_SADE);
            let kulma = osuma.clamp(-1.0, 1.0) * SUURIN_KULMA;
            self.nopeus = (self.vauhti * kulma.sin(), -self.vauhti * kulma.cos());
            tapahtumat.push(Tapahtuma::Maila);
        } else if self.pallo.1 - PALLON_SADE > KORKEUS {
            self.elamat -= 1;
            if self.elamat == 0 {
                self.tila = Tila::Ohi;
                tapahtumat.push(Tapahtuma::Ohi);
            } else {
                self.kiinnita_pallo();
                tapahtumat.push(Tapahtuma::Menetys);
            }
        }
        tapahtumat
    }

    fn osuu_mailaan(&self) -> bool {
        let (x, y) = self.pallo;
        self.nopeus.1 > 0.0
            && y + PALLON_SADE >= MAILA_Y
            && y - PALLON_SADE < MAILA_Y + MAILA_K
            && (x - self.maila_x).abs() <= MAILA_L / 2.0 + PALLON_SADE
    }

    /// Ensimmäinen ehjä tiili pallon neliön alla.
    fn osuva_tiili(&self) -> Option<(usize, usize)> {
        let (x, y) = self.pallo;
        let rivit = (y - PALLON_SADE - TIILET_Y) / TIILI_K..(y + PALLON_SADE - TIILET_Y) / TIILI_K;
        let sarakkeet = (x - PALLON_SADE) / TIILI_L..(x + PALLON_SADE) / TIILI_L;
        let viimeinen = |arvo: f32| arvo.ceil() as i32;
        for rivi in rivit.start.floor() as i32..viimeinen(rivit.end) {
            for sarake in sarakkeet.start.floor() as i32..viimeinen(sarakkeet.end) {
                if !(0..RIVEJA as i32).contains(&rivi) || !(0..SARAKKEITA as i32).contains(&sarake)
                {
                    continue;
                }
                let paikka = (rivi as usize, sarake as usize);
                if self.tiilet[paikka.0][paikka.1] > 0 {
                    return Some(paikka);
                }
            }
        }
        None
    }

    fn osu_tiileen(&mut self, (rivi, sarake): (usize, usize), tapahtumat: &mut Vec<Tapahtuma>) {
        let tiili = &mut self.tiilet[rivi][sarake];
        *tiili -= 1;
        let rikki = *tiili == 0;
        self.pisteet += PISTEET_OSUMASTA * (self.kentta as u32 + 1);
        tapahtumat.push(Tapahtuma::Tiili {
            rivi,
            sarake,
            rikki,
        });

        let uusi = (self.vauhti + VAUHTI_TIILTA_KOHTI).min(SUURIN_VAUHTI);
        self.nopeus = (
            self.nopeus.0 * uusi / self.vauhti,
            self.nopeus.1 * uusi / self.vauhti,
        );
        self.vauhti = uusi;

        if self.tiilia_jaljella() > 0 {
            return;
        }
        if self.kentta + 1 == KENTTIA {
            self.tila = Tila::Voitto;
            tapahtumat.push(Tapahtuma::Voitto);
        } else {
            self.kentta += 1;
            self.tiilet = kentan_tiilet(self.kentta);
            self.kiinnita_pallo();
            tapahtumat.push(Tapahtuma::KenttaLapi);
        }
    }
}
