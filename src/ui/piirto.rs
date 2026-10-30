use std::f32::consts::{PI, TAU};

use laivanupotus::{KOKO, Laiva, Laivasto, Laukaustulos, Ruutu, SARAKKEET, Vaikeus};

use super::kaynnistin::{Kaynnistin, PELIT};
use super::peli::{Peli, TULOS_VIIVE_MS, Tehoste, Vaihe};
use super::vektori::{self, Kangas, kehapiste, varjosta};
use super::{fontti, vektorifontti};

pub const LEVEYS: usize = 960;
pub const KORKEUS: usize = 768;

// Paletti: C64:n siniset ja Amiga Workbench 1.3:n käyttöliittymävärit.
const REUNA: u32 = 0x7C70DA;
const TAUSTA: u32 = 0x352879;
const TAUSTA_TUMMA: u32 = 0x120A38;
const VESI: u32 = 0x0B1650;
const AALTO: u32 = 0x24408C;
const RUUDUKKO: u32 = 0x3A62B8;
const VALKOINEN: u32 = 0xFFFFFF;
const MUSTA: u32 = 0x000000;
const WB_SININEN: u32 = 0x0055AA;
const WB_ORANSSI: u32 = 0xFF8800;
const RUNKO: u32 = 0x5C6B78;
const KANSI: u32 = 0x8A98A4;
const TORNI: u32 = 0xC8D2DA;
const LAIVA_REUNA: u32 = 0xE8F4FF;
const OSUMA: u32 = 0xB53A2E;
const OSUMA_TUMMA: u32 = 0x4A0E0A;
const HUTI: u32 = 0x6ABFC6;
const KELTAINEN: u32 = 0xEEEE77;
const TAIVAS_YLA: u32 = 0x0A0626;
const TAIVAS_ALA: u32 = 0xD8508C;
const MERI_YLA: u32 = 0x18206E;
const MERI_ALA: u32 = 0x05081E;
const MERI_VIIVA: u32 = 0x4FA8E0;

const MERKKI: i32 = 3 * fontti::LEVEYS;
const RIVIN_KORKEUS: i32 = 24;
const SOLU: i32 = 33;
const LAUDAN_KOKO: i32 = SOLU * KOKO as i32;
const LAUTA_Y: i32 = 99;
const VASTUSTAJA_X: i32 = 90;
const OMA_X: i32 = 570;
const OTSIKKO_Y: i32 = 48;
const TILASTO_Y: i32 = 441;
const TILASTO_SISENNYS: i32 = 48;
const TILASTO_PALKKI_X: i32 = 14 * MERKKI;
const PALKIN_PALA: i32 = 18;
const PANEELI_Y: i32 = 603;
const PANEELI_K: i32 = 156;
const PANEELI_RIVI_Y: i32 = PANEELI_Y + 9;
const PANEELI_RIVEJA: usize = 6;
const TEKSTI_X: i32 = 24;
const RIVILLE_MERKKEJA: usize = 38;
const NAPPI_Y: i32 = PANEELI_RIVI_Y + 5 * RIVIN_KORKEUS - 6;
const NAPPI_K: i32 = 30;
const NAPPI_X: i32 = 30;
const AANI_PAALLA: &str = "Ääni: päällä";
const AANI_POIS: &str = "Ääni: pois";
// Mahtuu pidemmän tekstin mukaan, jottei kytkin muuta kokoaan.
const AANI_L: i32 = 12 * MERKKI + 24;
const AANI_X: i32 = LEVEYS as i32 - 12 - AANI_L;
const AANI_Y: i32 = 6;
const AANI_K: i32 = 30;

const KAISTOJA: i32 = 16;
const VILKKU_MS: u32 = 400;
const TEHOSTE_KESTO_MS: f32 = super::peli::TEHOSTE_KESTO_MS as f32;

// Alkuruudun maisema
const MAISEMA_REUNA: i32 = 24;
const HORISONTTI_Y: i32 = 195;
const MAISEMA_ALA_Y: i32 = 348;
const OHJE_Y: i32 = 360;
const MERI_RIVEJA: i32 = 10;
const MERI_SARAKKEITA: i32 = 14;
const LAIVAN_KIERTO: f32 = 0.0006;
const LAIVAN_KOKO: f32 = 33.0;

// (merkin leveys, merkin korkeus, askel) vektorifontille
const SUURI: (i32, i32, i32) = (30, 45, 39);
const OTSIKKO: (i32, i32, i32) = (15, 21, 21);
const TULOS: (i32, i32, i32) = (12, 18, 18);
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

// Käynnistimen pelikortit
const KORTTI_X: i32 = 48;
const KORTTI_L: i32 = LEVEYS as i32 - 2 * KORTTI_X;
const KORTTI_Y: i32 = 372;
const KORTTI_K: i32 = 84;
const KORTTI_ASKEL: i32 = KORTTI_K + 12;
const KORTTI_SISENNYS: i32 = 18;

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

const TASO_OTSIKKO: &str = "Taso:";
const TASO_Y: i32 = 672;
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

pub fn aanikytkimella(x: i32, y: i32) -> bool {
    (AANI_X..AANI_X + AANI_L).contains(&x) && (AANI_Y..AANI_Y + AANI_K).contains(&y)
}

#[cfg(test)]
pub fn aanikytkimen_keskipiste() -> (i32, i32) {
    (AANI_X + AANI_L / 2, AANI_Y + AANI_K / 2)
}

fn napin_leveys(teksti: &str) -> i32 {
    teksti.chars().count() as i32 * MERKKI + 24
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
fn sekoita(a: u32, b: u32, osuus: f32) -> u32 {
    let (a, b) = (a as i32, b as i32);
    let kanava = |siirto: i32| {
        let (x, y) = (a >> siirto & 0xFF, b >> siirto & 0xFF);
        ((x + ((y - x) as f32 * osuus) as i32) as u32) << siirto
    };
    kanava(16) | kanava(8) | kanava(0)
}

pub fn piirra_kaynnistin(kaynnistin: &Kaynnistin, kuva: &mut [u32]) {
    if let Some(peli) = &kaynnistin.peli {
        piirra(peli, kuva);
        return;
    }
    let mut p = Piirtaja {
        k: Kangas(kuva),
        kello: kaynnistin.kello_ms,
    };
    p.valikko(kaynnistin.valittu);
    p.aanikytkin(kaynnistin.aanet_paalla);
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

struct Piirtaja<'a> {
    k: Kangas<'a>,
    kello: u32,
}

impl Piirtaja<'_> {
    fn vilkku(&self) -> bool {
        (self.kello / VILKKU_MS).is_multiple_of(2)
    }

    /// Bittikarttateksti kolminkertaisena: 40 merkkiä riville kuten C64:ssä.
    fn teksti(&mut self, x: i32, y: i32, teksti: &str, vari: u32) {
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
    fn vektoriteksti(
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
    fn liuku(&mut self, y0: i32, y1: i32, yla: u32, ala: u32) {
        let kaistan_k = ((y1 - y0) / KAISTOJA).max(1);
        let mut y = y0;
        while y < y1 {
            let osuus = (y - y0) as f32 / (y1 - y0) as f32;
            let vari = sekoita(yla, ala, osuus);
            self.k
                .laatikko(0, y, LEVEYS as i32, kaistan_k.min(y1 - y), vari);
            y += kaistan_k;
        }
    }

    fn valikko(&mut self, valittu: usize) {
        self.maisema("**** PELIKONE ****");
        for (i, kortti) in PELIT.iter().enumerate() {
            let y = KORTTI_Y + i as i32 * KORTTI_ASKEL;
            let (tausta, nimi_vari, kuvaus_vari) = if i == valittu {
                (WB_ORANSSI, MUSTA, MUSTA)
            } else if kortti.valmis {
                (VALKOINEN, WB_SININEN, MUSTA)
            } else {
                (TAUSTA_TUMMA, REUNA, REUNA)
            };
            self.k.laatikko(KORTTI_X, y, KORTTI_L, KORTTI_K, tausta);
            self.k
                .laatikko(KORTTI_X + 3, y + KORTTI_K, KORTTI_L, 3, MUSTA);
            self.k
                .laatikko(KORTTI_X + KORTTI_L, y + 3, 3, KORTTI_K, MUSTA);
            let x = KORTTI_X + KORTTI_SISENNYS;
            let nimi = format!("{} {}", i + 1, kortti.nimi);
            self.vektoriteksti(x, y + 15, &nimi, OTSIKKO, nimi_vari, true);
            self.teksti(x, y + 48, kortti.kuvaus, kuvaus_vari);
        }
        let mut y = KORTTI_Y + PELIT.len() as i32 * KORTTI_ASKEL + 12;
        for rivi in [
            "Nuolet, numero tai hiiri: valitse.",
            "Välilyönti tai klikkaus: pelaa.",
            "M: äänet päälle tai pois.",
            "Esc: lopeta.",
        ] {
            self.teksti(KORTTI_X, y, rivi, VALKOINEN);
            y += RIVIN_KORKEUS;
        }
        if self.vilkku() {
            self.k.laatikko(KORTTI_X, y, MERKKI, MERKKI, REUNA);
        }
    }

    fn alkuruutu(&mut self, vaikeus: Vaikeus) {
        self.maisema("**** LAIVANUPOTUS ****");

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
        y = TASO_Y + NAPPI_K + 6;
        self.teksti(48, y, "Paina välilyöntiä tai klikkaa.", REUNA);
        if self.vilkku() {
            self.k.laatikko(48 + 30 * MERKKI, y, MERKKI, MERKKI, REUNA);
        }
    }

    /// Alkuruudun ja käynnistimen yhteinen tausta: maisema, otsikko ja C64:n reuna.
    fn maisema(&mut self, otsikko: &str) {
        self.liuku(0, HORISONTTI_Y, TAIVAS_YLA, TAIVAS_ALA);
        self.tahdet();
        self.liuku(HORISONTTI_Y, MAISEMA_ALA_Y, MERI_YLA, MERI_ALA);
        self.meriruudukko();
        let keinunta = (self.kello as f32 * 0.002).sin() * 3.0;
        let malli = vektori::taistelulaiva(RUNKO, KANSI, TORNI);
        self.k.malli(
            &malli,
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

    fn tahdet(&mut self) {
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
    fn meriruudukko(&mut self) {
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

    fn maiseman_otsikko(&mut self, teksti: &str) {
        let x = (LEVEYS as i32 - teksti.chars().count() as i32 * SUURI.2) / 2;
        self.vektoriteksti(x + 3, 48, teksti, SUURI, TAUSTA, true);
        self.vektoriteksti(x, 45, teksti, SUURI, KELTAINEN, true);
    }

    fn pelinakyma(&mut self, peli: &Peli) {
        self.liuku(0, KORKEUS as i32, TAUSTA_TUMMA, TAUSTA);
        let (l, k) = (LEVEYS as i32, KORKEUS as i32);
        self.k.kehys(0, 0, l, k, REUNA);
        self.k.kehys(1, 1, l - 2, k - 2, REUNA);
        self.otsikkopalkki(peli.vaikeus);

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

    fn otsikkopalkki(&mut self, vaikeus: Vaikeus) {
        self.k.laatikko(6, 6, LEVEYS as i32 - 12, 30, VALKOINEN);
        self.teksti(18, 9, "Laivanupotus", WB_SININEN);
        let nimi = vaikeus.nimi();
        let nimi_x = AANI_X - 12 - nimi.chars().count() as i32 * MERKKI;
        self.teksti(nimi_x, 9, nimi, WB_SININEN);
        for y in [9, 15, 21, 27] {
            self.k.laatikko(324, y, nimi_x - 336, 3, WB_SININEN);
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

    fn rajahdys(&mut self, keski: (i32, i32), ika: f32, sateita: i32, koko: f32) {
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

    fn aanikytkin(&mut self, paalla: bool) {
        let (tausta, teksti) = if paalla {
            (VALKOINEN, AANI_PAALLA)
        } else {
            (WB_ORANSSI, AANI_POIS)
        };
        self.k.laatikko(AANI_X, AANI_Y, AANI_L, AANI_K, tausta);
        self.k.kehys(AANI_X, AANI_Y, AANI_L, AANI_K, MUSTA);
        self.teksti(AANI_X + 12, AANI_Y + 3, teksti, MUSTA);
    }

    fn napit(&mut self) {
        for (_, teksti, x) in NAPIT {
            self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
        }
    }

    fn nappi(&mut self, x: i32, y: i32, teksti: &str, tausta: u32, vari: u32) {
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
