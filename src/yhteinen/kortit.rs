//! Pelikorttien piirto. Kortti piirretään aina täysikokoiselle ruudulle.

use std::f32::consts::TAU;

use klassikot8000::kortit::{KUNINGAS, Kortti, Maa};

use super::piirto::*;
use super::vektori::{Tahko, kehapiste, tahko4};

pub const KORTTI_L: i32 = 150;
pub const KORTTI_K: i32 = 210;

/// Yksi pelimerkki kolmiulotteiseen malliin. `taso` kertoo sen paikan pinossa alhaalta lukien.
pub fn pelimerkki(x: f32, z: f32, taso: usize, vari: u32, tahkot: &mut Vec<Tahko>) {
    const KULMIA: usize = 8;
    const SADE: f32 = 1.2;
    const KORKEUS: f32 = 0.35;
    let (y0, y1) = (taso as f32 * KORKEUS, (taso + 1) as f32 * KORKEUS);
    let piste = |kulma: usize, y: f32| {
        let (sin, cos) = (kulma as f32 / KULMIA as f32 * TAU).sin_cos();
        [x + SADE * cos, y, z + SADE * sin]
    };
    for kulma in 0..KULMIA {
        let seuraava = (kulma + 1) % KULMIA;
        tahkot.push(tahko4(
            piste(kulma, y0),
            piste(seuraava, y0),
            piste(seuraava, y1),
            piste(kulma, y1),
            vari,
        ));
    }
    // Kansi kolmesta nelikulmiosta, koska tahkossa ei ole kahdeksaa kulmaa.
    for [a, b, c, d] in [[0, 1, 2, 3], [0, 3, 4, 7], [4, 5, 6, 7]] {
        tahkot.push(tahko4(
            piste(a, y1),
            piste(b, y1),
            piste(c, y1),
            piste(d, y1),
            vari,
        ));
    }
}

fn maan_vari(maa: Maa) -> u32 {
    if maa.on_punainen() { OSUMA } else { MUSTA }
}

fn arvon_nimi(arvo: u8) -> String {
    match arvo {
        1 => "A".to_string(),
        11 => "J".to_string(),
        12 => "Q".to_string(),
        KUNINGAS => "K".to_string(),
        _ => arvo.to_string(),
    }
}

/// Maakuvioiden paikat kortin keskialueella, -1..1 kumpaankin suuntaan.
fn kuviot(arvo: u8) -> &'static [(f32, f32)] {
    const A: f32 = 0.55;
    const B: f32 = 0.33;
    const C: f32 = 0.66;
    match arvo {
        2 => &[(0.0, -1.0), (0.0, 1.0)],
        3 => &[(0.0, -1.0), (0.0, 0.0), (0.0, 1.0)],
        4 => &[(-A, -1.0), (A, -1.0), (-A, 1.0), (A, 1.0)],
        5 => &[(-A, -1.0), (A, -1.0), (0.0, 0.0), (-A, 1.0), (A, 1.0)],
        6 => &[
            (-A, -1.0),
            (A, -1.0),
            (-A, 0.0),
            (A, 0.0),
            (-A, 1.0),
            (A, 1.0),
        ],
        7 => &[
            (-A, -1.0),
            (A, -1.0),
            (0.0, -0.5),
            (-A, 0.0),
            (A, 0.0),
            (-A, 1.0),
            (A, 1.0),
        ],
        8 => &[
            (-A, -1.0),
            (A, -1.0),
            (0.0, -0.5),
            (-A, 0.0),
            (A, 0.0),
            (0.0, 0.5),
            (-A, 1.0),
            (A, 1.0),
        ],
        9 => &[
            (-A, -1.0),
            (A, -1.0),
            (-A, -B),
            (A, -B),
            (0.0, 0.0),
            (-A, B),
            (A, B),
            (-A, 1.0),
            (A, 1.0),
        ],
        10 => &[
            (-A, -1.0),
            (A, -1.0),
            (0.0, -C),
            (-A, -B),
            (A, -B),
            (-A, B),
            (A, B),
            (0.0, C),
            (-A, 1.0),
            (A, 1.0),
        ],
        _ => &[],
    }
}

impl Piirtaja<'_> {
    /// Selkäpuoli on Amiga-sinistä ruudutusta.
    pub fn kortin_tausta(&mut self, x: i32, y: i32) {
        self.k.laatikko(x + 3, y + 3, KORTTI_L, KORTTI_K, MUSTA);
        self.k.laatikko(x, y, KORTTI_L, KORTTI_K, WB_SININEN);
        self.k.kehys(x, y, KORTTI_L, KORTTI_K, MUSTA);
        self.k
            .kehys(x + 6, y + 6, KORTTI_L - 12, KORTTI_K - 12, VALKOINEN);
        const RUUTU: i32 = 12;
        for rivi in 0..(KORTTI_K - 24) / RUUTU {
            for sarake in 0..(KORTTI_L - 24) / RUUTU {
                if (rivi + sarake) % 2 == 0 {
                    let (rx, ry) = (x + 12 + sarake * RUUTU, y + 12 + rivi * RUUTU);
                    self.k.laatikko(rx, ry, RUUTU, RUUTU, REUNA);
                }
            }
        }
    }

    pub fn pelikortti(&mut self, x: i32, y: i32, kortti: Kortti) {
        let vari = maan_vari(kortti.maa);
        self.k.laatikko(x + 3, y + 3, KORTTI_L, KORTTI_K, MUSTA);
        self.k.laatikko(x, y, KORTTI_L, KORTTI_K, VALKOINEN);
        self.k.kehys(x, y, KORTTI_L, KORTTI_K, MUSTA);

        let nimi = arvon_nimi(kortti.arvo);
        self.vektoriteksti(x + 9, y + 8, &nimi, OTSIKKO, vari, true);
        self.maakuvio(kortti.maa, (x + KORTTI_L - 21, y + 18), 12.0, vari);

        let keski = (x + KORTTI_L / 2, y + 42 + (KORTTI_K - 42) / 2);
        match kortti.arvo {
            1 => self.maakuvio(kortti.maa, keski, 42.0, vari),
            11..=KUNINGAS => {
                // Kuvakortit ovat kehystetty kirjain, kuten vanhojen koneiden pasiansseissa.
                let (kx, ky) = (x + 21, y + 42);
                self.k
                    .laatikko(kx, ky, KORTTI_L - 42, KORTTI_K - 57, KELTAINEN);
                self.k.kehys(kx, ky, KORTTI_L - 42, KORTTI_K - 57, vari);
                self.vektoriteksti(keski.0 - SUURI.0 / 2, ky + 12, &nimi, SUURI, vari, true);
                self.maakuvio(kortti.maa, (keski.0, ky + 105), 21.0, vari);
            }
            arvo => {
                for &(px, py) in kuviot(arvo) {
                    let kohta = (keski.0 + (px * 54.0) as i32, keski.1 + (py * 66.0) as i32);
                    self.maakuvio(kortti.maa, kohta, 15.0, vari);
                }
            }
        }
    }

    fn taytetty_ympyra(&mut self, keski: (i32, i32), sade: f32, vari: u32) {
        const SIVUJA: usize = 12;
        let mut pisteet = [(0, 0); SIVUJA];
        for (i, piste) in pisteet.iter_mut().enumerate() {
            let kulma = i as f32 / SIVUJA as f32 * TAU;
            *piste = kehapiste(keski, sade, kulma);
        }
        self.k.monikulmio(&pisteet, vari);
    }

    /// Maat kootaan ympyröistä ja monikulmioista, joten ne skaalautuvat mihin tahansa kokoon.
    pub fn maakuvio(&mut self, maa: Maa, keski: (i32, i32), koko: f32, vari: u32) {
        let piste = |x: f32, y: f32| (keski.0 + (x * koko) as i32, keski.1 + (y * koko) as i32);
        let sade = koko * 0.5;
        match maa {
            Maa::Ruutu => {
                let kulmat = [
                    piste(0.0, -1.0),
                    piste(0.8, 0.0),
                    piste(0.0, 1.0),
                    piste(-0.8, 0.0),
                ];
                self.k.monikulmio(&kulmat, vari);
            }
            Maa::Hertta => {
                self.taytetty_ympyra(piste(-0.47, -0.4), sade, vari);
                self.taytetty_ympyra(piste(0.47, -0.4), sade, vari);
                let karki = [piste(-0.95, -0.2), piste(0.95, -0.2), piste(0.0, 1.0)];
                self.k.monikulmio(&karki, vari);
            }
            Maa::Pata => {
                self.taytetty_ympyra(piste(-0.47, 0.2), sade, vari);
                self.taytetty_ympyra(piste(0.47, 0.2), sade, vari);
                let karki = [piste(-0.95, 0.0), piste(0.95, 0.0), piste(0.0, -1.0)];
                self.k.monikulmio(&karki, vari);
                let varsi = [piste(0.0, 0.3), piste(0.35, 1.0), piste(-0.35, 1.0)];
                self.k.monikulmio(&varsi, vari);
            }
            Maa::Risti => {
                self.taytetty_ympyra(piste(0.0, -0.5), sade, vari);
                self.taytetty_ympyra(piste(-0.52, 0.2), sade, vari);
                self.taytetty_ympyra(piste(0.52, 0.2), sade, vari);
                let varsi = [piste(0.0, -0.2), piste(0.35, 1.0), piste(-0.35, 1.0)];
                self.k.monikulmio(&varsi, vari);
            }
        }
    }
}
