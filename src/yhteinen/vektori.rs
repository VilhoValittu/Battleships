//! Ohjelmistopiirron alkeet: viivat, täytetyt monikulmiot ja kevyt 3D-malli.

use std::f32::consts::TAU;

use super::piirto::{KORKEUS, LEVEYS};

const SUURIN_LEIKKAUS: usize = 16;
const KAMERAN_ETAISYYS: f32 = 16.0;
const KALLISTUS: f32 = 0.24;
const VALO: [f32; 3] = [0.41, 0.82, -0.41];

pub struct Kangas<'a> {
    kuva: &'a mut [u32],
    pub leveys: usize,
    pub korkeus: usize,
}

impl<'a> Kangas<'a> {
    /// Klassinen 960x768-kuva, johon vanhat näkymät piirtävät.
    pub fn new(kuva: &'a mut [u32]) -> Self {
        Self::kokoinen(kuva, LEVEYS, KORKEUS)
    }

    pub fn kokoinen(kuva: &'a mut [u32], leveys: usize, korkeus: usize) -> Self {
        Self {
            kuva,
            leveys,
            korkeus,
        }
    }
}

impl Kangas<'_> {
    pub fn laatikko(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        let (a, b) = (
            x.max(0) as usize,
            (x + l).clamp(0, self.leveys as i32) as usize,
        );
        if a >= b {
            return;
        }
        for yy in y.max(0)..(y + k).min(self.korkeus as i32) {
            let rivi = yy as usize * self.leveys;
            self.kuva[rivi + a..rivi + b].fill(vari);
        }
    }

    pub fn kehys(&mut self, x: i32, y: i32, l: i32, k: i32, vari: u32) {
        self.laatikko(x, y, l, 1, vari);
        self.laatikko(x, y + k - 1, l, 1, vari);
        self.laatikko(x, y, 1, k, vari);
        self.laatikko(x + l - 1, y, 1, k, vari);
    }

    pub fn piste(&mut self, x: i32, y: i32, vari: u32) {
        if (0..self.leveys as i32).contains(&x) && (0..self.korkeus as i32).contains(&y) {
            self.kuva[y as usize * self.leveys + x as usize] = vari;
        }
    }

    pub fn viiva(&mut self, alku: (i32, i32), loppu: (i32, i32), vari: u32) {
        let (mut x, mut y) = alku;
        let (dx, dy) = ((loppu.0 - x).abs(), -(loppu.1 - y).abs());
        let (askel_x, askel_y) = ((loppu.0 - x).signum(), (loppu.1 - y).signum());
        let mut virhe = dx + dy;
        loop {
            self.piste(x, y, vari);
            if (x, y) == loppu {
                return;
            }
            let kaksi = 2 * virhe;
            if kaksi >= dy {
                virhe += dy;
                x += askel_x;
            }
            if kaksi <= dx {
                virhe += dx;
                y += askel_y;
            }
        }
    }

    pub fn paksu_viiva(&mut self, alku: (i32, i32), loppu: (i32, i32), vari: u32) {
        self.viiva(alku, loppu, vari);
        self.viiva((alku.0 + 1, alku.1), (loppu.0 + 1, loppu.1), vari);
        self.viiva((alku.0, alku.1 + 1), (loppu.0, loppu.1 + 1), vari);
    }

    /// Täyttää monikulmion vaakaviivoin, parillis-pariton-säännöllä.
    pub fn monikulmio(&mut self, pisteet: &[(i32, i32)], vari: u32) {
        let ylin = pisteet.iter().map(|p| p.1).min().unwrap_or(0).max(0);
        let alin = pisteet
            .iter()
            .map(|p| p.1)
            .max()
            .unwrap_or(0)
            .min(self.korkeus as i32 - 1);
        for y in ylin..=alin {
            let mut leikkaukset = [0; SUURIN_LEIKKAUS];
            let mut maara = 0;
            for (i, &a) in pisteet.iter().enumerate() {
                let b = pisteet[(i + 1) % pisteet.len()];
                if (a.1 <= y) != (b.1 <= y) && maara < SUURIN_LEIKKAUS {
                    leikkaukset[maara] = a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1);
                    maara += 1;
                }
            }
            leikkaukset[..maara].sort_unstable();
            for pari in leikkaukset[..maara].chunks_exact(2) {
                self.laatikko(pari[0], y, pari[1] - pari[0] + 1, 1, vari);
            }
        }
    }

    pub fn murtoviiva(&mut self, pisteet: &[(i32, i32)], vari: u32) {
        for pari in pisteet.windows(2) {
            self.viiva(pari[0], pari[1], vari);
        }
    }

    /// Ympyrä on säännöllinen monikulmio, kuten aikansa vektoripeleissä.
    pub fn ympyra(&mut self, keski: (i32, i32), sade: f32, vari: u32) {
        const SIVUJA: i32 = 16;
        let mut edellinen = kehapiste(keski, sade, 0.0);
        for i in 1..=SIVUJA {
            let seuraava = kehapiste(keski, sade, i as f32 / SIVUJA as f32 * TAU);
            self.viiva(edellinen, seuraava, vari);
            edellinen = seuraava;
        }
    }

    /// Piirtää mallin kaukaisimmasta tahkosta lähimpään, jolloin kukin peittää taakseen
    /// jäävät. Tasainen varjostus neljällä tasolla ja kirkkaat reunaviivat.
    pub fn malli(
        &mut self,
        tahkot: &[Tahko],
        kulma: f32,
        keski: (i32, i32),
        mitta: f32,
        reuna: u32,
    ) {
        let (sin_y, cos_y) = kulma.sin_cos();
        let (sin_x, cos_x) = KALLISTUS.sin_cos();
        let kierra = |p: [f32; 3]| {
            let x = p[0] * cos_y + p[2] * sin_y;
            let z = p[2] * cos_y - p[0] * sin_y;
            [x, p[1] * cos_x + z * sin_x, z * cos_x - p[1] * sin_x]
        };

        let mut jarjestys: Vec<(f32, &Tahko, [[f32; 3]; 5])> = tahkot
            .iter()
            .map(|tahko| {
                let kierretyt = tahko.pisteet.map(kierra);
                let n = tahko.maara as f32;
                let syvyys = kierretyt[..tahko.maara].iter().map(|p| p[2]).sum::<f32>() / n;
                (syvyys, tahko, kierretyt)
            })
            .collect();
        jarjestys.sort_by(|a, b| b.0.total_cmp(&a.0));

        for (_, tahko, kierretyt) in jarjestys {
            let mut ruudulla = [(0, 0); 5];
            for (kohde, p) in ruudulla.iter_mut().zip(kierretyt) {
                let syvyys = mitta * KAMERAN_ETAISYYS / (p[2] + KAMERAN_ETAISYYS);
                *kohde = (
                    keski.0 + (p[0] * syvyys) as i32,
                    keski.1 - (p[1] * syvyys) as i32,
                );
            }
            let ruudulla = &ruudulla[..tahko.maara];
            self.monikulmio(ruudulla, varjosta(tahko.vari, valaistus(&kierretyt)));
            for (i, &a) in ruudulla.iter().enumerate() {
                self.viiva(a, ruudulla[(i + 1) % ruudulla.len()], reuna);
            }
        }
    }
}

pub fn kehapiste(keski: (i32, i32), sade: f32, kulma: f32) -> (i32, i32) {
    let (sin, cos) = kulma.sin_cos();
    (keski.0 + (cos * sade) as i32, keski.1 + (sin * sade) as i32)
}

/// Väri kerrottuna kertoimella, jolla kukin kanava tummenee.
pub fn varjosta(vari: u32, kerroin: f32) -> u32 {
    let kanava = |siirto: u32| (((vari >> siirto & 0xFF) as f32 * kerroin) as u32).min(255);
    kanava(16) << 16 | kanava(8) << 8 | kanava(0)
}

fn valaistus(p: &[[f32; 3]; 5]) -> f32 {
    let a = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
    let b = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
    let n = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let pituus = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(0.001);
    let osuvuus = (n[0] * VALO[0] + n[1] * VALO[1] + n[2] * VALO[2]).abs() / pituus;
    0.4 + (osuvuus * 3.0).round() * 0.2
}

#[derive(Clone, Copy)]
pub struct Tahko {
    pisteet: [[f32; 3]; 5],
    maara: usize,
    vari: u32,
}

fn tahko4(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3], vari: u32) -> Tahko {
    Tahko {
        pisteet: [a, b, c, d, d],
        maara: 4,
        vari,
    }
}

/// Laatikon kyljet ja katto, pohjaa ei koskaan näy.
pub fn kuutio(x: [f32; 2], y: [f32; 2], z: [f32; 2], vari: u32, tahkot: &mut Vec<Tahko>) {
    let p = |i: usize, j: usize, k: usize| [x[i], y[j], z[k]];
    tahkot.push(tahko4(p(0, 1, 0), p(1, 1, 0), p(1, 1, 1), p(0, 1, 1), vari));
    tahkot.push(tahko4(p(0, 0, 0), p(1, 0, 0), p(1, 1, 0), p(0, 1, 0), vari));
    tahkot.push(tahko4(p(0, 0, 1), p(1, 0, 1), p(1, 1, 1), p(0, 1, 1), vari));
    tahkot.push(tahko4(p(0, 0, 0), p(0, 0, 1), p(0, 1, 1), p(0, 1, 0), vari));
    tahkot.push(tahko4(p(1, 0, 0), p(1, 0, 1), p(1, 1, 1), p(1, 1, 0), vari));
}

/// Alkuruudun taistelulaiva. Keula osoittaa +x-suuntaan, vesiraja on y = 0.
pub fn taistelulaiva(runko: u32, kansi: u32, torni: u32) -> Vec<Tahko> {
    let kansi_p = [
        [-5.0, 0.7, -0.8],
        [-5.0, 0.7, 0.8],
        [2.0, 0.7, 1.1],
        [5.5, 0.9, 0.0],
        [2.0, 0.7, -1.1],
    ];
    let vesi_p = [
        [-4.6, 0.0, -0.6],
        [-4.6, 0.0, 0.6],
        [2.0, 0.0, 0.9],
        [4.8, 0.0, 0.0],
        [2.0, 0.0, -0.9],
    ];
    let mut tahkot = vec![Tahko {
        pisteet: kansi_p,
        maara: 5,
        vari: kansi,
    }];
    for i in 0..5 {
        let j = (i + 1) % 5;
        tahkot.push(tahko4(kansi_p[i], kansi_p[j], vesi_p[j], vesi_p[i], runko));
    }
    // komentosilta, masto ja savupiippu
    kuutio([-0.8, 1.0], [0.7, 1.7], [-0.55, 0.55], torni, &mut tahkot);
    kuutio([-0.3, 0.7], [1.7, 2.4], [-0.35, 0.35], torni, &mut tahkot);
    kuutio([0.1, 0.25], [2.4, 3.3], [-0.07, 0.07], kansi, &mut tahkot);
    kuutio([-2.0, -1.4], [0.7, 1.6], [-0.3, 0.3], runko, &mut tahkot);
    // tykkitornit putkineen
    for (x, suunta) in [(2.2, 1.0), (-3.4, -1.0)] {
        kuutio(
            [x - 0.45, x + 0.45],
            [0.7, 1.1],
            [-0.4, 0.4],
            torni,
            &mut tahkot,
        );
        let putki_x = if suunta > 0.0 {
            [x + 0.4, x + 1.7]
        } else {
            [x - 1.7, x - 0.4]
        };
        kuutio(putki_x, [0.85, 0.97], [-0.06, 0.06], kansi, &mut tahkot);
    }
    tahkot
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tyhja() -> Vec<u32> {
        vec![0; LEVEYS * KORKEUS]
    }

    fn pikseli(kuva: &[u32], x: usize, y: usize) -> u32 {
        kuva[y * LEVEYS + x]
    }

    #[test]
    fn viiva_ulottuu_paasta_paahan_ja_kuvan_ulkopuoli_ohitetaan() {
        let mut kuva = tyhja();
        let mut k = Kangas::new(&mut kuva);
        k.viiva((10, 10), (40, 25), 1);
        k.viiva((-50, -50), (LEVEYS as i32 + 50, KORKEUS as i32 + 50), 2);
        assert_eq!(pikseli(&kuva, 10, 10), 1);
        assert_eq!(pikseli(&kuva, 40, 25), 1);
    }

    #[test]
    fn monikulmio_tayttaa_sisaosan_muttei_ulkopuolta() {
        let mut kuva = tyhja();
        Kangas::new(&mut kuva).monikulmio(&[(10, 10), (30, 10), (20, 30)], 7);
        assert_eq!(pikseli(&kuva, 20, 15), 7);
        assert_eq!(pikseli(&kuva, 11, 25), 0);
    }

    #[test]
    fn monikulmio_kuvan_reunan_yli_ei_kaada() {
        let mut kuva = tyhja();
        Kangas::new(&mut kuva).monikulmio(&[(-20, -20), (LEVEYS as i32 + 20, 0), (0, 2000)], 7);
        assert_eq!(pikseli(&kuva, 0, 0), 7);
    }

    #[test]
    fn malli_piirtyy_kaikissa_kulmissa() {
        let malli = taistelulaiva(0x111111, 0x222222, 0x333333);
        for i in 0..32 {
            let mut kuva = tyhja();
            Kangas::new(&mut kuva).malli(&malli, i as f32 * 0.2, (480, 225), 33.0, 0xFFFFFF);
            assert!(kuva.iter().filter(|&&p| p != 0).count() > 500);
        }
    }

    #[test]
    fn varjostus_ei_ylita_kanavaa() {
        assert_eq!(varjosta(0xFF8040, 2.0), 0xFFFF80);
        assert_eq!(varjosta(0xFF8040, 0.5), 0x7F4020);
    }
}
