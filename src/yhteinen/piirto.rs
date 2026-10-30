use std::f32::consts::TAU;

use super::vektori::{self, Kangas, kehapiste, varjosta};
use super::{fontti, vektorifontti};

pub const LEVEYS: usize = 960;
pub const KORKEUS: usize = 768;

/// Ikkunan todellinen koko. `LEVEYS` ja `KORKEUS` ovat klassisten näkymien kuva, joka
/// skaalataan ruudun korkuiseksi keskelle; laidat jäävät C64:n reunaväriin.
pub const RUUDUN_LEVEYS: usize = 1920;
pub const RUUDUN_KORKEUS: usize = 1080;
// Suhde 1080 / 768 = 45 / 32.
const SKAALA_OSOITTAJA: usize = 45;
const SKAALA_NIMITTAJA: usize = 32;
const SKAALATTU_LEVEYS: usize = LEVEYS * SKAALA_OSOITTAJA / SKAALA_NIMITTAJA;
const SKAALATTU_X: usize = (RUUDUN_LEVEYS - SKAALATTU_LEVEYS) / 2;

// Paletti: C64:n siniset ja Amiga Workbench 1.3:n käyttöliittymävärit.
pub const REUNA: u32 = 0x7C70DA;
pub const TAUSTA: u32 = 0x352879;
pub const TAUSTA_TUMMA: u32 = 0x120A38;
pub const VESI: u32 = 0x0B1650;
pub const AALTO: u32 = 0x24408C;
pub const RUUDUKKO: u32 = 0x3A62B8;
pub const VALKOINEN: u32 = 0xFFFFFF;
pub const MUSTA: u32 = 0x000000;
pub const WB_SININEN: u32 = 0x0055AA;
pub const WB_ORANSSI: u32 = 0xFF8800;
pub const RUNKO: u32 = 0x5C6B78;
pub const KANSI: u32 = 0x8A98A4;
pub const TORNI: u32 = 0xC8D2DA;
pub const LAIVA_REUNA: u32 = 0xE8F4FF;
pub const OSUMA: u32 = 0xB53A2E;
pub const OSUMA_TUMMA: u32 = 0x4A0E0A;
pub const HUTI: u32 = 0x6ABFC6;
pub const KELTAINEN: u32 = 0xEEEE77;
pub const TAIVAS_YLA: u32 = 0x0A0626;
pub const TAIVAS_ALA: u32 = 0xD8508C;
pub const MERI_YLA: u32 = 0x18206E;
pub const MERI_ALA: u32 = 0x05081E;
pub const MERI_VIIVA: u32 = 0x4FA8E0;

pub const MERKKI: i32 = 3 * fontti::LEVEYS;
pub const RIVIN_KORKEUS: i32 = 24;
pub const PALKIN_PALA: i32 = 18;
pub const PANEELI_Y: i32 = 603;
pub const PANEELI_K: i32 = 156;
pub const PANEELI_RIVI_Y: i32 = PANEELI_Y + 9;
pub const PANEELI_RIVEJA: usize = 6;
pub const TEKSTI_X: i32 = 24;
pub const RIVILLE_MERKKEJA: usize = 38;
pub const NAPPI_Y: i32 = PANEELI_RIVI_Y + 5 * RIVIN_KORKEUS - 6;
pub const NAPPI_K: i32 = 30;
pub const NAPPI_X: i32 = 30;
const AANI_PAALLA: &str = "Ääni: päällä";
const AANI_POIS: &str = "Ääni: pois";
// Mahtuu pidemmän tekstin mukaan, jottei kytkin muuta kokoaan.
const AANI_L: i32 = 12 * MERKKI + 24;
const AANI_X: i32 = LEVEYS as i32 - 12 - AANI_L;
const AANI_Y: i32 = 6;
const AANI_K: i32 = 30;

const KAISTOJA: i32 = 16;
const VILKKU_MS: u32 = 400;

// Alkuruutujen viimeinen rivi: "Paina välilyöntiä..."
pub const ALOITUS_Y: i32 = 708;

// Alkuruudun maisema
const MAISEMA_REUNA: i32 = 24;
const HORISONTTI_Y: i32 = 195;
const MAISEMA_ALA_Y: i32 = 348;
pub const OHJE_Y: i32 = 360;
const MERI_RIVEJA: i32 = 10;
const MERI_SARAKKEITA: i32 = 14;
const LAIVAN_KIERTO: f32 = 0.0006;
const LAIVAN_KOKO: f32 = 33.0;

// (merkin leveys, merkin korkeus, askel) vektorifontille
pub const SUURI: (i32, i32, i32) = (30, 45, 39);
pub const OTSIKKO: (i32, i32, i32) = (15, 21, 21);
pub const TULOS: (i32, i32, i32) = (12, 18, 18);

pub fn aanikytkimella(x: i32, y: i32) -> bool {
    (AANI_X..AANI_X + AANI_L).contains(&x) && (AANI_Y..AANI_Y + AANI_K).contains(&y)
}

#[cfg(test)]
pub fn aanikytkimen_keskipiste() -> (i32, i32) {
    (AANI_X + AANI_L / 2, AANI_Y + AANI_K / 2)
}

pub fn napin_leveys(teksti: &str) -> i32 {
    teksti.chars().count() as i32 * MERKKI + 24
}

/// Venyttää klassisen kuvan ruudulle. Lähimmän pikselin menetelmä säilyttää
/// kovat reunat, ja sama rivi kopioidaan laskematta uudelleen.
pub fn skaalaa_ruudulle(kuva: &[u32], ruutu: &mut [u32]) {
    let lahde_x: [u16; SKAALATTU_LEVEYS] =
        std::array::from_fn(|x| (x * SKAALA_NIMITTAJA / SKAALA_OSOITTAJA) as u16);
    let mut edellinen = None;
    for y in 0..RUUDUN_KORKEUS {
        let rivi = y * RUUDUN_LEVEYS;
        let lahde_y = y * SKAALA_NIMITTAJA / SKAALA_OSOITTAJA;
        if edellinen == Some(lahde_y) {
            ruutu.copy_within(rivi - RUUDUN_LEVEYS..rivi, rivi);
            continue;
        }
        edellinen = Some(lahde_y);
        let kohde = &mut ruutu[rivi..rivi + RUUDUN_LEVEYS];
        kohde.fill(REUNA);
        let lahde = &kuva[lahde_y * LEVEYS..(lahde_y + 1) * LEVEYS];
        for (pikseli, &x) in kohde[SKAALATTU_X..].iter_mut().zip(&lahde_x) {
            *pikseli = lahde[x as usize];
        }
    }
}

/// Ruudun pikseli klassisen kuvan koordinaateiksi. Laidoilla x jää kuvan ulkopuolelle.
pub fn ruudulta_kuvaan(x: i32, y: i32) -> (i32, i32) {
    let (osoittaja, nimittaja) = (SKAALA_OSOITTAJA as i32, SKAALA_NIMITTAJA as i32);
    (
        ((x - SKAALATTU_X as i32) * nimittaja).div_euclid(osoittaja),
        (y * nimittaja).div_euclid(osoittaja),
    )
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

/// Sekoitetaan kaksi väriä, `osuus` 0 antaa ensimmäisen ja 1 toisen.
pub fn sekoita(a: u32, b: u32, osuus: f32) -> u32 {
    let (a, b) = (a as i32, b as i32);
    let kanava = |siirto: i32| {
        let (x, y) = (a >> siirto & 0xFF, b >> siirto & 0xFF);
        ((x + ((y - x) as f32 * osuus) as i32) as u32) << siirto
    };
    kanava(16) | kanava(8) | kanava(0)
}

pub struct Piirtaja<'a> {
    pub k: Kangas<'a>,
    pub kello: u32,
}

impl Piirtaja<'_> {
    pub fn vilkku(&self) -> bool {
        (self.kello / VILKKU_MS).is_multiple_of(2)
    }

    /// Bittikarttateksti kolminkertaisena: 40 merkkiä riville kuten C64:ssä.
    pub fn teksti(&mut self, x: i32, y: i32, teksti: &str, vari: u32) {
        for (i, merkki) in teksti.chars().enumerate() {
            let vasen = x + i as i32 * MERKKI;
            for (rivi, bitit) in fontti::kuvio(merkki).iter().enumerate() {
                for sarake in 0..8 {
                    if bitit >> sarake & 1 != 0 {
                        self.k
                            .laatikko(vasen + sarake * 3, y + rivi as i32 * 3, 3, 3, vari);
                    }
                }
            }
        }
    }

    /// Vektoriteksti. `koko` on (merkin leveys, korkeus, askel) pikseleinä.
    pub fn vektoriteksti(
        &mut self,
        x: i32,
        y: i32,
        teksti: &str,
        koko: (i32, i32, i32),
        vari: u32,
        paksu: bool,
    ) {
        for (i, merkki) in teksti.chars().enumerate() {
            let vasen = x + i as i32 * koko.2;
            for osa in vektorifontti::kuvio(merkki) {
                for pari in osa.windows(2) {
                    let piste = |(px, py): (i8, i8)| {
                        (
                            vasen + px as i32 * koko.0 / vektorifontti::LEVEYS,
                            y + py as i32 * koko.1 / vektorifontti::KORKEUS,
                        )
                    };
                    let (a, b) = (piste(pari[0]), piste(pari[1]));
                    if paksu {
                        self.k.paksu_viiva(a, b, vari);
                    } else {
                        self.k.viiva(a, b, vari);
                    }
                }
            }
        }
    }

    /// Amigan copper-palkkien tapaan väri vaihtuu kaistoittain.
    pub fn liuku(&mut self, y0: i32, y1: i32, yla: u32, ala: u32) {
        let kaistan_k = ((y1 - y0) / KAISTOJA).max(1);
        let mut y = y0;
        while y < y1 {
            let osuus = (y - y0) as f32 / (y1 - y0) as f32;
            let vari = sekoita(yla, ala, osuus);
            self.k
                .laatikko(0, y, self.k.leveys as i32, kaistan_k.min(y1 - y), vari);
            y += kaistan_k;
        }
    }

    /// Alkuruudun ja käynnistimen yhteinen tausta: maisema, otsikko ja C64:n reuna.
    pub fn maisema(&mut self, otsikko: &str, malli: &[vektori::Tahko]) {
        self.liuku(0, HORISONTTI_Y, TAIVAS_YLA, TAIVAS_ALA);
        self.tahdet();
        self.liuku(HORISONTTI_Y, MAISEMA_ALA_Y, MERI_YLA, MERI_ALA);
        self.meriruudukko();
        let keinunta = (self.kello as f32 * 0.002).sin() * 3.0;
        self.k.malli(
            malli,
            self.kello as f32 * LAIVAN_KIERTO,
            (LEVEYS as i32 / 2, HORISONTTI_Y + 33 + keinunta as i32),
            LAIVAN_KOKO,
            LAIVA_REUNA,
        );
        self.maiseman_otsikko(otsikko);

        // C64:n leveä reuna rajaa maiseman ja tekstin
        let (l, k) = (LEVEYS as i32, KORKEUS as i32);
        self.k.laatikko(0, 0, l, MAISEMA_REUNA, REUNA);
        self.k
            .laatikko(0, k - MAISEMA_REUNA, l, MAISEMA_REUNA, REUNA);
        self.k.laatikko(0, 0, MAISEMA_REUNA, k, REUNA);
        self.k
            .laatikko(l - MAISEMA_REUNA, 0, MAISEMA_REUNA, k, REUNA);
        self.k.laatikko(
            MAISEMA_REUNA,
            MAISEMA_ALA_Y,
            l - 2 * MAISEMA_REUNA,
            k - MAISEMA_ALA_Y - MAISEMA_REUNA,
            TAUSTA,
        );
        self.k.laatikko(
            MAISEMA_REUNA,
            MAISEMA_ALA_Y,
            l - 2 * MAISEMA_REUNA,
            2,
            REUNA,
        );
    }

    pub fn tahdet(&mut self) {
        // kiinteä siemen: tähdet pysyvät paikoillaan ruudusta toiseen
        let mut siemen = 0x2F6Eu32;
        for _ in 0..60 {
            siemen = siemen.wrapping_mul(0x9E37_79B1).rotate_left(11);
            let x = (siemen >> 8) as i32 % LEVEYS as i32;
            let y = (siemen & 0xFF) as i32 * (HORISONTTI_Y - 45) / 256;
            self.k.piste(x, y, sekoita(TAIVAS_YLA, VALKOINEN, 0.8));
        }
    }

    /// Perspektiiviruudukko, jonka vaakaviivat vyöryvät katsojaa kohti.
    pub fn meriruudukko(&mut self) {
        let korkeus = (MAISEMA_ALA_Y - HORISONTTI_Y) as f32;
        let vaihe = (self.kello % 1600) as f32 / 1600.0;
        for i in 0..MERI_RIVEJA {
            let t = (i as f32 + vaihe) / MERI_RIVEJA as f32;
            let y = HORISONTTI_Y + (korkeus * t * t) as i32;
            let vari = sekoita(MERI_YLA, MERI_VIIVA, 0.25 + 0.75 * t);
            self.k.laatikko(0, y, LEVEYS as i32, 1, vari);
        }
        let keski = LEVEYS as i32 / 2;
        for i in -MERI_SARAKKEITA..=MERI_SARAKKEITA {
            let ala_x = keski + i * 135;
            self.k.viiva(
                (keski + i * 12, HORISONTTI_Y),
                (ala_x, MAISEMA_ALA_Y),
                sekoita(MERI_YLA, MERI_VIIVA, 0.55),
            );
        }
    }

    pub fn maiseman_otsikko(&mut self, teksti: &str) {
        let x = (LEVEYS as i32 - teksti.chars().count() as i32 * SUURI.2) / 2;
        self.vektoriteksti(x + 3, 48, teksti, SUURI, TAUSTA, true);
        self.vektoriteksti(x, 45, teksti, SUURI, KELTAINEN, true);
    }

    pub fn otsikkopalkki(&mut self, otsikko: &str, nimi: &str) {
        self.k
            .laatikko(6, 6, self.k.leveys as i32 - 12, 30, VALKOINEN);
        self.teksti(18, 9, otsikko, WB_SININEN);
        let aani_x = self.k.leveys as i32 - 12 - AANI_L;
        let nimi_x = aani_x - 12 - nimi.chars().count() as i32 * MERKKI;
        self.teksti(nimi_x, 9, nimi, WB_SININEN);
        let raita_x = 36 + otsikko.chars().count() as i32 * MERKKI;
        for y in [9, 15, 21, 27] {
            self.k
                .laatikko(raita_x, y, nimi_x - 12 - raita_x, 3, WB_SININEN);
        }
    }

    pub fn rajahdys(&mut self, keski: (i32, i32), ika: f32, sateita: i32, koko: f32) {
        let vari = if ika < 0.3 {
            VALKOINEN
        } else if ika < 0.6 {
            KELTAINEN
        } else {
            WB_ORANSSI
        };
        for i in 0..sateita {
            // sirpaleet lentävät eri pituisina, jottei räjähdys ole ympyrä
            let kerroin = 0.6 + ((i * 37) % 11) as f32 / 25.0;
            let kulma = i as f32 / sateita as f32 * TAU;
            let ulko = kehapiste(keski, ika * koko * kerroin + 6.0, kulma);
            let sisa = kehapiste(keski, ika * koko * kerroin * 0.55, kulma);
            self.k.viiva(sisa, ulko, vari);
        }
        self.k
            .ympyra(keski, ika * koko * 0.8, varjosta(vari, 1.0 - ika * 0.6));
    }

    pub fn aanikytkin(&mut self, paalla: bool) {
        let (tausta, teksti) = if paalla {
            (VALKOINEN, AANI_PAALLA)
        } else {
            (WB_ORANSSI, AANI_POIS)
        };
        let x = self.k.leveys as i32 - 12 - AANI_L;
        self.k.laatikko(x, AANI_Y, AANI_L, AANI_K, tausta);
        self.k.kehys(x, AANI_Y, AANI_L, AANI_K, MUSTA);
        self.teksti(x + 12, AANI_Y + 3, teksti, MUSTA);
    }

    pub fn nappi(&mut self, x: i32, y: i32, teksti: &str, tausta: u32, vari: u32) {
        let leveys = napin_leveys(teksti);
        self.k.laatikko(x, y, leveys, NAPPI_K, tausta);
        self.k
            .laatikko(x + 3, y + NAPPI_K - 3, leveys - 3, 3, MUSTA);
        self.k
            .laatikko(x + leveys - 3, y + 3, 3, NAPPI_K - 3, MUSTA);
        self.teksti(x + 12, y + 3, teksti, vari);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn rivitys_katkaisee_sanavalista_ja_kasittelee_aakkoset() {
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
    fn skaalaus_tayttaa_ruudun_ja_hiiri_osuu_samaan_pikseliin() {
        let kuva: Vec<u32> = (0..LEVEYS * KORKEUS).map(|i| i as u32 + 1).collect();
        let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
        skaalaa_ruudulle(&kuva, &mut ruutu);
        assert_eq!(ruutu[0], REUNA);
        assert_eq!(ruutu[RUUDUN_LEVEYS * RUUDUN_KORKEUS - 1], REUNA);
        for (x, y) in [(285, 0), (700, 300), (1634, 1079), (1000, 1079)] {
            let (kx, ky) = ruudulta_kuvaan(x, y);
            let pikseli = ruutu[y as usize * RUUDUN_LEVEYS + x as usize];
            assert_eq!(pikseli, kuva[ky as usize * LEVEYS + kx as usize], "{x},{y}");
        }
        assert!(ruudulta_kuvaan(0, 0).0 < 0);
        assert!(ruudulta_kuvaan(1700, 0).0 >= LEVEYS as i32);
    }
}
