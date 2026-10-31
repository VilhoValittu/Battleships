//! 2048-pelin näkymät.

use klassikot8000::luvut::{Lukulauta, Paikka, SIVU, Suunta, TAVOITE};

use super::peli::{KIPINA_MS, LIUKU_MS, Luvut, Nappi, Vaihe};
use crate::yhteinen::kieli::{kaanna, muotoile};
use crate::yhteinen::piirto::*;
use crate::yhteinen::vektori::{Kangas, Tahko, kuutio};

const POMPAHDUS_MS: u32 = 140;

const RUUTU: i32 = 132;
const LAATTA: i32 = RUUTU - 12;
const LAUDAN_SIVU: i32 = RUUTU * SIVU as i32;
const RUUDUT_X: i32 = 30;
const RUUDUT_Y: i32 = 48;
const SARAKE_X: i32 = RUUDUT_X + LAUDAN_SIVU + 30;
const TILASTON_ASKEL: i32 = 87;
const TAVOITE_Y: i32 = RUUDUT_Y + 4 * TILASTON_ASKEL;
const ASKELIA_TAVOITTEESEEN: i32 = 11;
const NUOLEN_KOKO: i32 = 15;
const PARHAAT_Y: i32 = TAVOITE_Y + 66;
const PARHAAT_L: i32 = LEVEYS as i32 - 12 - SARAKE_X;

const OHJE: &str = "luvut.ohje";
const EI_LIIKU: &str = "luvut.ei_liiku";
const UUSI_PELI: &str = "luvut.uusi_peli";
const JATKA: &str = "luvut.jatka";
const VOITTO: &str = "luvut.voitto";
const TAPPIO: &str = "yhteinen.peli_paattyi";

const NAPIT: [(Nappi, &str, i32); 3] = [
    (Nappi::Uudestaan, "yhteinen.uudestaan", NAPPI_X),
    (
        Nappi::Valikko,
        "yhteinen.valikkoon",
        NAPPI_X + 13 * MERKKI + 48,
    ),
    (
        Nappi::Jatka,
        "luvut.jatka_nappi",
        NAPPI_X + 24 * MERKKI + 84,
    ),
];

const ALKU_OSAT: [&str; 5] = [
    "luvut.tervetuloa",
    "yhteinen.ohjeet",
    "luvut.ohje_1",
    "luvut.ohje_2",
    "luvut.ohje_3",
];
const ALKU_RIVIT: [&str; 4] = [
    "luvut.nappain_nuolet",
    "luvut.nappain_hiiri",
    "yhteinen.aanet_kytkin",
    "yhteinen.takaisin",
];

/// Laudan pikseli kertoo suunnan: se puoli keskipisteestä, jota kohti on pisin matka.
pub fn suunta_pisteesta(x: i32, y: i32) -> Option<Suunta> {
    let sisalla = (RUUDUT_X..RUUDUT_X + LAUDAN_SIVU).contains(&x)
        && (RUUDUT_Y..RUUDUT_Y + LAUDAN_SIVU).contains(&y);
    if !sisalla {
        return None;
    }
    let (dx, dy) = (
        x - RUUDUT_X - LAUDAN_SIVU / 2,
        y - RUUDUT_Y - LAUDAN_SIVU / 2,
    );
    Some(if dx.abs() >= dy.abs() {
        if dx > 0 { Suunta::Oikea } else { Suunta::Vasen }
    } else if dy > 0 {
        Suunta::Alas
    } else {
        Suunta::Ylos
    })
}

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

/// Piste laudan reunan keskellä: sinne piirretään myös hiiren suuntanuoli.
pub fn laudan_reuna(suunta: Suunta) -> (i32, i32) {
    let (kx, ky) = (RUUDUT_X + LAUDAN_SIVU / 2, RUUDUT_Y + LAUDAN_SIVU / 2);
    let matka = LAUDAN_SIVU / 2 - NUOLEN_KOKO;
    match suunta {
        Suunta::Ylos => (kx, ky - matka),
        Suunta::Alas => (kx, ky + matka),
        Suunta::Vasen => (kx - matka, ky),
        Suunta::Oikea => (kx + matka, ky),
    }
}

/// Laatan tausta ja numero. Värit kulkevat C64-sinisestä Amigan oranssiin.
fn laatan_varit(arvo: u32) -> (u32, u32) {
    match arvo {
        2 => (0x7C70DA, VALKOINEN),
        4 => (0x4FA8E0, MUSTA),
        8 => (HUTI, MUSTA),
        16 => (0x70B050, MUSTA),
        32 => (KELTAINEN, MUSTA),
        64 => (WB_ORANSSI, MUSTA),
        128 => (TAIVAS_ALA, VALKOINEN),
        256 => (OSUMA, VALKOINEN),
        512 => (0x8B3FC0, VALKOINEN),
        1024 => (WB_SININEN, VALKOINEN),
        2048 => (0xFFD700, MUSTA),
        _ => (MUSTA, KELTAINEN),
    }
}

/// Merkin koko numeron pituuden mukaan: (leveys, korkeus, askel).
fn numeron_koko(pituus: usize) -> (i32, i32, i32) {
    match pituus {
        0..=2 => (33, 54, 42),
        3 => (27, 42, 33),
        4 => (21, 33, 27),
        5 => (16, 27, 21),
        _ => (13, 21, 18),
    }
}

fn ruudun_keskipiste((rivi, sarake): Paikka) -> (i32, i32) {
    (
        RUUDUT_X + sarake as i32 * RUUTU + RUUTU / 2,
        RUUDUT_Y + rivi as i32 * RUUTU + RUUTU / 2,
    )
}

/// Montako tuplausta luvusta 2 tarvitaan. Palkki täyttyy samassa tahdissa.
fn tuplauksia(arvo: u32) -> i32 {
    arvo.max(1).trailing_zeros() as i32
}

/// Alkuruudun malli: neljä laattaa kasvavina pinoina. Mitat kuin laivalla.
fn lukupino(varit: [u32; 4]) -> Vec<Tahko> {
    let mut tahkot = Vec::new();
    for (i, vari) in varit.into_iter().enumerate() {
        let x = -4.6 + i as f32 * 2.4;
        let korkeus = 0.8 + i as f32 * 0.8;
        kuutio([x, x + 2.0], [0.0, korkeus], [-1.0, 1.0], vari, &mut tahkot);
    }
    tahkot
}

pub fn piirra(luvut: &Luvut, kuva: &mut [u32]) {
    let mut p = Piirtaja {
        k: Kangas::new(kuva),
        kello: luvut.kello_ms,
    };
    if luvut.vaihe == Vaihe::Alku {
        p.luvut_alkuruutu(luvut);
    } else {
        p.luvut_nakyma(luvut);
    }
    p.aanikytkin(luvut.aanet_paalla);
}

impl Piirtaja<'_> {
    fn luvut_alkuruutu(&mut self, luvut: &Luvut) {
        let jatkuu = luvut.jatkuu;
        let varit = [2, 8, 64, 2048].map(|arvo| laatan_varit(arvo).0);
        self.maisema(kaanna("luvut.banneri"), &lukupino(varit));
        let otsikko = listan_otsikko(luvut.netti_parhaat.is_some());
        self.parhaat_alkuruudulla(otsikko, luvut.nakyva_lista());
        let y = self.alkuohje(&ALKU_OSAT, &ALKU_RIVIT);
        if jatkuu {
            self.teksti(48, y, kaanna(UUSI_PELI), KELTAINEN);
        }
        self.aloituskehote(if jatkuu { JATKA } else { ALOITUS });
    }

    fn luvut_nakyma(&mut self, luvut: &Luvut) {
        self.liuku(0, KORKEUS as i32, TAUSTA_TUMMA, TAUSTA);
        let (l, k) = (LEVEYS as i32, KORKEUS as i32);
        self.k.kehys(0, 0, l, k, REUNA);
        self.k.kehys(1, 1, l - 2, k - 2, REUNA);
        self.otsikkopalkki(kaanna("peli.luvut"), "");

        self.luku_lauta();
        self.laatat(luvut);
        if luvut.vaihe == Vaihe::Pelissa
            && let Some(suunta) = luvut.hiiren_suunta
        {
            self.suuntanuoli(suunta);
        }
        self.luku_tilasto(luvut);
        self.luku_paneeli(luvut);
    }

    /// Lauta on samaa aaltoilevaa vettä kuin laivanupotuksessa.
    fn luku_lauta(&mut self) {
        self.k
            .laatikko(RUUDUT_X, RUUDUT_Y, LAUDAN_SIVU, LAUDAN_SIVU, VESI);
        const VALI: i32 = 33;
        for rivi in 0..SIVU as i32 * 2 {
            let y = RUUDUT_Y + rivi * RUUTU / 2 + RUUTU / 4;
            let vaihe = self.kello as f32 * 0.0025 + rivi as f32 * 1.4;
            let mut pisteet = [(0, 0); 17];
            for (i, piste) in pisteet.iter_mut().enumerate() {
                let korkeus = (vaihe + i as f32 * 0.9).sin() * 3.5;
                *piste = (RUUDUT_X + i as i32 * VALI, y + korkeus as i32);
            }
            self.k.murtoviiva(&pisteet, AALTO);
        }
        for i in 0..=SIVU as i32 {
            let viiva = (i * RUUTU).min(LAUDAN_SIVU - 1);
            self.k
                .laatikko(RUUDUT_X + viiva, RUUDUT_Y, 1, LAUDAN_SIVU, RUUDUKKO);
            self.k
                .laatikko(RUUDUT_X, RUUDUT_Y + viiva, LAUDAN_SIVU, 1, RUUDUKKO);
        }
        self.k.kehys(
            RUUDUT_X - 1,
            RUUDUT_Y - 1,
            LAUDAN_SIVU + 2,
            LAUDAN_SIVU + 2,
            REUNA,
        );
    }

    fn laatat(&mut self, luvut: &Luvut) {
        let kulunut = luvut
            .viime_siirto
            .as_ref()
            .map(|(_, alku)| self.kello.saturating_sub(*alku));
        if let (Some((siirto, _)), Some(kulunut)) = (&luvut.viime_siirto, kulunut)
            && kulunut < LIUKU_MS
        {
            let t = kulunut as f32 / LIUKU_MS as f32;
            for liike in &siirto.liikkeet {
                let (ax, ay) = ruudun_keskipiste(liike.mista);
                let (bx, by) = ruudun_keskipiste(liike.mihin);
                let keski = (
                    ax + ((bx - ax) as f32 * t) as i32,
                    ay + ((by - ay) as f32 * t) as i32,
                );
                self.laatta(keski, LAATTA, liike.arvo);
            }
            return;
        }

        let pompahdus = kulunut.map_or(1.0, |kulunut| {
            ((kulunut.saturating_sub(LIUKU_MS)) as f32 / POMPAHDUS_MS as f32).min(1.0)
        });
        for rivi in 0..SIVU {
            for sarake in 0..SIVU {
                let paikka = (rivi, sarake);
                let arvo = luvut.lauta.arvo(paikka);
                if arvo == 0 {
                    continue;
                }
                let siirto = luvut.viime_siirto.as_ref().map(|(siirto, _)| siirto);
                // uusi luku kasvaa esiin, yhdistynyt paisuu ja palautuu
                let kerroin = match siirto {
                    Some(s) if s.uusi == paikka => 0.3 + 0.7 * pompahdus,
                    Some(s) if s.yhdistyneet.contains(&paikka) => 1.0 + 0.12 * (1.0 - pompahdus),
                    _ => 1.0,
                };
                let sivu = (LAATTA as f32 * kerroin) as i32;
                self.laatta(ruudun_keskipiste(paikka), sivu, arvo);
            }
        }
        if let (Some((siirto, _)), Some(kulunut)) = (&luvut.viime_siirto, kulunut) {
            let ika = (kulunut - LIUKU_MS) as f32 / KIPINA_MS as f32;
            for &paikka in &siirto.yhdistyneet {
                let arvo = luvut.lauta.arvo(paikka);
                let koko = if arvo >= 128 { 105.0 } else { 66.0 };
                self.rajahdys(ruudun_keskipiste(paikka), ika, 12, koko);
            }
        }
    }

    /// Amiga-napin tapaan kohoava laatta. `sivu` muuttuu animaatioissa.
    fn laatta(&mut self, keski: (i32, i32), sivu: i32, arvo: u32) {
        let (tausta, teksti_vari) = laatan_varit(arvo);
        let (x, y) = (keski.0 - sivu / 2, keski.1 - sivu / 2);
        self.k.laatikko(x + 3, y + 3, sivu, sivu, MUSTA);
        self.k.laatikko(x, y, sivu, sivu, tausta);
        self.k
            .laatikko(x, y, sivu, 3, sekoita(tausta, VALKOINEN, 0.5));
        self.k
            .laatikko(x, y, 3, sivu, sekoita(tausta, VALKOINEN, 0.5));

        let teksti = arvo.to_string();
        let pituus = teksti.chars().count();
        let (l, k, askel) = numeron_koko(pituus);
        let mittaa = |arvo: i32| arvo * sivu / LAATTA;
        let koko = (mittaa(l), mittaa(k), mittaa(askel));
        let leveys = pituus as i32 * koko.2 - (koko.2 - koko.0);
        let (tx, ty) = (keski.0 - leveys / 2, keski.1 - koko.1 / 2);
        // Kaksi kerrosta, jotta numero erottuu isosta laatasta.
        self.vektoriteksti(tx, ty, &teksti, koko, teksti_vari, true);
        self.vektoriteksti(tx + 1, ty + 1, &teksti, koko, teksti_vari, true);
    }

    fn suuntanuoli(&mut self, suunta: Suunta) {
        let (x, y) = laudan_reuna(suunta);
        let vari = if self.vilkku() { KELTAINEN } else { WB_ORANSSI };
        let n = NUOLEN_KOKO;
        let kolmio = match suunta {
            Suunta::Ylos => [(x, y - n), (x + n, y + n), (x - n, y + n)],
            Suunta::Alas => [(x, y + n), (x + n, y - n), (x - n, y - n)],
            Suunta::Vasen => [(x - n, y), (x + n, y - n), (x + n, y + n)],
            Suunta::Oikea => [(x + n, y), (x - n, y - n), (x - n, y + n)],
        };
        self.k.monikulmio(&kolmio, vari);
        for (i, &a) in kolmio.iter().enumerate() {
            self.k.viiva(a, kolmio[(i + 1) % kolmio.len()], MUSTA);
        }
    }

    fn luku_tilasto(&mut self, luvut: &Luvut) {
        let lauta: &Lukulauta = &luvut.lauta;
        for (i, (otsikko, arvo)) in [
            (kaanna("yhteinen.pisteet"), lauta.pisteet),
            (kaanna("yhteinen.paras"), luvut.paras),
            (kaanna("luvut.suurin"), lauta.suurin()),
            (kaanna("yhteinen.siirrot"), luvut.siirtoja),
        ]
        .into_iter()
        .enumerate()
        {
            let y = RUUDUT_Y + i as i32 * TILASTON_ASKEL;
            self.vektoriteksti(SARAKE_X, y, otsikko, OTSIKKO, KELTAINEN, false);
            let numero = arvo.to_string();
            self.vektoriteksti(SARAKE_X, y + 30, &numero, OTSIKKO, VALKOINEN, true);
        }

        // Palkki kertoo matkan tavoitteeseen samoilla väreillä kuin laatat.
        let otsikko = muotoile("luvut.tavoite", &[&TAVOITE]);
        self.teksti(SARAKE_X, TAVOITE_Y, &otsikko, KELTAINEN);
        let saavutettu = tuplauksia(lauta.suurin());
        for i in 1..=ASKELIA_TAVOITTEESEEN {
            let x = SARAKE_X + (i - 1) * PALKIN_PALA;
            let vari = if i <= saavutettu {
                laatan_varit(1 << i).0
            } else {
                KANSI
            };
            let (y, korkeus) = if i <= saavutettu {
                (TAVOITE_Y + 30, 24)
            } else {
                (TAVOITE_Y + 42, 12)
            };
            self.k.laatikko(x, y, PALKIN_PALA - 3, korkeus, vari);
        }
        self.parhaat_lista(
            SARAKE_X,
            PARHAAT_Y,
            PARHAAT_L,
            TULOS,
            listan_otsikko(luvut.netti_parhaat.is_some()),
            luvut.nakyva_lista(),
        );
    }

    fn luku_paneeli(&mut self, luvut: &Luvut) {
        let leveys = LEVEYS as i32 - 12;
        self.k.laatikko(6, PANEELI_Y, leveys, PANEELI_K, WB_SININEN);
        self.k.kehys(6, PANEELI_Y, leveys, PANEELI_K, VALKOINEN);
        self.teksti(TEKSTI_X, PANEELI_RIVI_Y, &luvut.viesti, VALKOINEN);

        let alarivi = PANEELI_RIVI_Y + (PANEELI_RIVEJA as i32 - 1) * RIVIN_KORKEUS;
        let otsikko_y = alarivi - RIVIN_KORKEUS - 12;
        match luvut.vaihe {
            Vaihe::Pelissa if luvut.ei_liiku => {
                self.teksti(TEKSTI_X, alarivi, kaanna(EI_LIIKU), WB_ORANSSI);
            }
            Vaihe::Pelissa => self.teksti(TEKSTI_X, alarivi, kaanna(OHJE), HUTI),
            Vaihe::Voitto => {
                self.vektoriteksti(TEKSTI_X, otsikko_y, kaanna(VOITTO), TULOS, KELTAINEN, true);
                for (_, teksti, x) in NAPIT {
                    self.nappi(x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
                }
            }
            Vaihe::Tappio => {
                self.vektoriteksti(TEKSTI_X, otsikko_y, kaanna(TAPPIO), TULOS, KELTAINEN, true);
                for (_, teksti, x) in &NAPIT[..2] {
                    self.nappi(*x, NAPPI_Y, teksti, VALKOINEN, WB_SININEN);
                }
            }
            Vaihe::Nimi => self.nimikysely(luvut.lauta.pisteet, &luvut.nimi),
            Vaihe::Alku => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::kieli::kaikilla_kielilla;

    #[test]
    fn laudan_neljannekset_antavat_suunnan_ja_ulkopuoli_ei() {
        for suunta in [Suunta::Ylos, Suunta::Alas, Suunta::Vasen, Suunta::Oikea] {
            let (x, y) = laudan_reuna(suunta);
            assert_eq!(suunta_pisteesta(x, y), Some(suunta));
        }
        assert_eq!(suunta_pisteesta(RUUDUT_X - 1, RUUDUT_Y), None);
        assert_eq!(suunta_pisteesta(RUUDUT_X, RUUDUT_Y + LAUDAN_SIVU), None);
    }

    #[test]
    fn napit_eivat_mene_paallekkain_ja_mahtuvat_ruudulle() {
        kaikilla_kielilla(|| {
            for pari in NAPIT.windows(2) {
                assert!(pari[0].2 + napin_leveys(pari[0].1) < pari[1].2);
            }
            let (_, teksti, x) = NAPIT[2];
            assert!(x + napin_leveys(teksti) < LEVEYS as i32);
        });
    }

    #[test]
    fn numerot_mahtuvat_laattaan_ja_tekstit_paneeliin() {
        for pituus in 1..=6 {
            let (l, _, askel) = numeron_koko(pituus);
            assert!(
                pituus as i32 * askel - (askel - l) <= LAATTA - 12,
                "{pituus}"
            );
        }
        kaikilla_kielilla(|| {
            for avain in [
                OHJE,
                EI_LIIKU,
                NIMI_OHJE,
                "luvut.jatketaan",
                "luvut.jatketaan_tallennettua",
            ] {
                let teksti = kaanna(avain);
                assert!(teksti.chars().count() <= RIVILLE_MERKKEJA, "{teksti}");
            }
            let siirto = muotoile("luvut.siirto_pisteet", &[&99999, &99999]);
            assert!(siirto.chars().count() <= RIVILLE_MERKKEJA, "{siirto}");
            assert!(kaanna(JATKA).chars().count() <= 36);
            assert!(kaanna(UUSI_PELI).chars().count() <= 36);
            assert!(kaanna(VOITTO).chars().count() as i32 * TULOS.2 < LEVEYS as i32 - TEKSTI_X);
        });
        // Tallennetun pelin rivi tulee näppäinohjeiden jälkeen.
        let rivit: Vec<&str> = ALKU_RIVIT.iter().copied().chain([UUSI_PELI]).collect();
        alkuohje_mahtuu(&ALKU_OSAT, &rivit, ALOITUS_Y);
    }

    #[test]
    fn koko_peli_piirtyy_kaatumatta_animaatioiden_aikana() {
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut luvut = Luvut::new(3);
        piirra(&luvut, &mut kuva);
        luvut.vaihe = Vaihe::Pelissa;
        let suunnat = [Suunta::Vasen, Suunta::Ylos, Suunta::Oikea, Suunta::Alas];
        for i in 0..400 {
            luvut.hiiren_suunta = Some(suunnat[i % 4]);
            luvut.siirra(suunnat[i % 4]);
            for _ in 0..3 {
                luvut.edista(70);
                piirra(&luvut, &mut kuva);
            }
        }
        luvut.vaihe = Vaihe::Nimi;
        piirra(&luvut, &mut kuva);
    }
}
