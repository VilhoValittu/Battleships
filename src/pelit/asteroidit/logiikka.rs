//! Asteroidien säännöt ilman ikkunaa, piirtoa tai syötettä. Mitat ovat kentän pikseleitä,
//! kenttä jatkuu reunojen yli, ja aika etenee kiinteinä askelina, joten sama siemen ja
//! ohjaus toistavat saman pelin.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::Arpa;

pub const LEVEYS: f32 = 660.0;
pub const KORKEUS: f32 = 540.0;
pub const ASKEL_MS: u32 = 5;
pub const ELAMIA: u32 = 3;
pub const ALUKSEN_SADE: f32 = 10.0;
pub const OMIA_AMMUKSIA: usize = 4;
pub const SUOJA_MS: u32 = 2500;
pub const ODOTUS_MS: u32 = 1500;
pub const AALLON_TAUKO_MS: u32 = 2000;
pub const LISAELAMAN_VALI: u32 = 10_000;
/// Myöhäinen aalto tuottaa pisteitä nopeasti, joten jokainen lisäalus maksaa edellistä enemmän.
pub const LISAELAMAN_KASVU: u32 = 5_000;
pub const PIENEN_LAUTASEN_RAJA: u32 = 5000;
/// Kuusi numeroa mahtuu sivupalkkiin, ja nettilista hylkää tätä suuremmat tulokset.
pub const SUURIN_PISTEMAARA: u32 = 999_999;

const ASKEL_S: f32 = ASKEL_MS as f32 / 1000.0;
const KAANTO: f32 = 4.5;
const KIIHTYVYYS: f32 = 320.0;
const KITKA: f32 = 0.35;
const SUURIN_VAUHTI: f32 = 380.0;
const NOKAN_MATKA: f32 = 14.0;
const AMMUKSEN_VAUHTI: f32 = 480.0;
const AMMUKSEN_IKA_MS: u32 = 1000;
const HYPYN_VALI_MS: u32 = 1000;
const KIVIA_ALUSSA: usize = 4;
const KIVIA_ENINTAAN: usize = 11;
// Kivi ei saa ilmestyä aluksen päälle.
const KIVEN_VALI_ALUKSESTA: f32 = 150.0;
const KIVEN_VAUHTI: (usize, usize) = (35, 70);
const KIVEN_VAUHTI_AALLOSTA: f32 = 4.0;
const KIVEN_SUURIN_VAUHTI: f32 = 170.0;
const SIRPALEEN_KIIHDYTYS: f32 = 1.3;
const LAUTASEN_VIIVE_MS: (usize, usize) = (15_000, 25_000);
// Kivien määrä ja vauhti tulevat vastaan, joten myöhemmillä aalloilla lautaset yltyvät.
const LAUTASEN_VIIVE_AALLOSTA_MS: u32 = 1000;
const LAUTASEN_VIIVE_VAHIN_MS: u32 = 6000;
const LAUTASEN_KAANTO_MS: u32 = 1200;
const LAUTASEN_LAUKAUS_MS: u32 = 1100;
const LAUTASEN_AMMUKSEN_VAUHTI: f32 = 300.0;
const LAUTASEN_AMMUKSEN_IKA_MS: u32 = 1400;
// Pieni lautanen tähtää, mutta ei aivan tarkasti: ±0,2 radiaania.
const TAHTAYKSEN_HAJONTA: usize = 40;

pub type Piste = (f32, f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Koko {
    Iso,
    Keski,
    Pieni,
}

impl Koko {
    pub fn sade(self) -> f32 {
        match self {
            Koko::Iso => 44.0,
            Koko::Keski => 22.0,
            Koko::Pieni => 11.0,
        }
    }

    pub fn pisteet(self) -> u32 {
        match self {
            Koko::Iso => 20,
            Koko::Keski => 50,
            Koko::Pieni => 100,
        }
    }

    fn sirpale(self) -> Option<Koko> {
        match self {
            Koko::Iso => Some(Koko::Keski),
            Koko::Keski => Some(Koko::Pieni),
            Koko::Pieni => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kivi {
    pub paikka: Piste,
    pub nopeus: Piste,
    pub koko: Koko,
    pub kulma: f32,
    pub pyorinta: f32,
    /// Ulkonäön numero: piirto valitsee tällä kiven muodon.
    pub muoto: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ammus {
    pub paikka: Piste,
    pub nopeus: Piste,
    pub jaljella_ms: u32,
    /// Tosi pelaajan ammukselle. Lautasen ammus lentää kivien läpi.
    pub oma: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lautanen {
    pub paikka: Piste,
    pub nopeus: Piste,
    /// Pieni lautanen on vaikeampi osua, tähtää ja antaa enemmän pisteitä.
    pub pieni: bool,
    jaljella_ms: u32,
    kaantyy_ms: u32,
    laukaus_ms: u32,
}

impl Lautanen {
    pub fn sade(&self) -> f32 {
        if self.pieni { 10.0 } else { 16.0 }
    }

    pub fn pisteet(&self) -> u32 {
        if self.pieni { 1000 } else { 200 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alus {
    pub paikka: Piste,
    pub nopeus: Piste,
    /// Radiaaneina, 0 on oikealle ja kulma kasvaa myötäpäivään.
    pub kulma: f32,
}

impl Alus {
    fn keskella() -> Self {
        Self {
            paikka: (LEVEYS / 2.0, KORKEUS / 2.0),
            nopeus: (0.0, 0.0),
            kulma: -FRAC_PI_2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ohjaus {
    /// -1 vastapäivään, 1 myötäpäivään, 0 ei kääntöä.
    pub kaanto: i8,
    pub tyonto: bool,
    /// Hiiren paikka kentällä. Alus kääntyy sitä kohti, kun `kaanto` on nolla.
    pub tahtain: Option<Piste>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tila {
    Lennossa,
    /// Alus tuhoutui, ja seuraava odottaa vuoroaan.
    Tuho,
    Ohi,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tapahtuma {
    Kivi { paikka: Piste, koko: Koko },
    LautanenTuhoutui { paikka: Piste },
    LautanenAmpui,
    Menetys { paikka: Piste },
    Palasi,
    Lisaelama,
    UusiAalto,
    Ohi { paikka: Piste },
}

pub struct Avaruus {
    pub alus: Alus,
    pub kivet: Vec<Kivi>,
    pub ammukset: Vec<Ammus>,
    pub lautanen: Option<Lautanen>,
    pub tila: Tila,
    pub elamat: u32,
    pub pisteet: u32,
    /// Nollasta alkava aallon numero.
    pub aalto: usize,
    /// Tosi, kun alus kiihdyttää: piirto näyttää liekin.
    pub tyontaa: bool,
    /// Suojan jäljellä oleva aika uuden aluksen alussa.
    pub suoja_ms: u32,
    pub lautasen_viive_ms: u32,
    odotus_ms: u32,
    aallon_viive_ms: u32,
    hyppy_ms: u32,
    seuraava_lisaelama: u32,
    lisaelaman_vali: u32,
}

fn arvo_vali(arpa: &mut Arpa, (ala, yla): (usize, usize)) -> usize {
    ala + arpa.alle(yla - ala)
}

fn lautasen_viive(aalto: usize, arpa: &mut Arpa) -> u32 {
    let lyhennys = (aalto as u32).saturating_mul(LAUTASEN_VIIVE_AALLOSTA_MS);
    (arvo_vali(arpa, LAUTASEN_VIIVE_MS) as u32)
        .saturating_sub(lyhennys)
        .max(LAUTASEN_VIIVE_VAHIN_MS)
}

fn arvo_kulma(arpa: &mut Arpa) -> f32 {
    arpa.alle(3600) as f32 / 3600.0 * TAU
}

fn suunnasta(kulma: f32, vauhti: f32) -> Piste {
    let (sin, cos) = kulma.sin_cos();
    (cos * vauhti, sin * vauhti)
}

fn liiku(paikka: &mut Piste, nopeus: Piste) {
    paikka.0 = (paikka.0 + nopeus.0 * ASKEL_S).rem_euclid(LEVEYS);
    paikka.1 = (paikka.1 + nopeus.1 * ASKEL_S).rem_euclid(KORKEUS);
}

/// Lyhin siirtymä pisteestä `a` pisteeseen `b`, kun kenttä jatkuu reunojen yli.
fn siirtyma(a: Piste, b: Piste) -> Piste {
    let lyhin = |ero: f32, koko: f32| {
        let ero = ero.rem_euclid(koko);
        if ero > koko / 2.0 { ero - koko } else { ero }
    };
    (lyhin(b.0 - a.0, LEVEYS), lyhin(b.1 - a.1, KORKEUS))
}

pub fn etaisyys(a: Piste, b: Piste) -> f32 {
    let (x, y) = siirtyma(a, b);
    x.hypot(y)
}

impl Avaruus {
    /// Peli alkaa annetusta aallosta.
    pub fn new(aalto: usize, arpa: &mut Arpa) -> Self {
        let mut peli = Self {
            alus: Alus::keskella(),
            kivet: Vec::new(),
            ammukset: Vec::new(),
            lautanen: None,
            tila: Tila::Lennossa,
            elamat: ELAMIA,
            pisteet: 0,
            aalto,
            tyontaa: false,
            suoja_ms: 0,
            lautasen_viive_ms: lautasen_viive(aalto, arpa),
            odotus_ms: 0,
            aallon_viive_ms: AALLON_TAUKO_MS,
            hyppy_ms: 0,
            seuraava_lisaelama: LISAELAMAN_VALI,
            lisaelaman_vali: LISAELAMAN_VALI,
        };
        peli.luo_aalto(arpa);
        peli
    }

    fn luo_aalto(&mut self, arpa: &mut Arpa) {
        let maara = (KIVIA_ALUSSA + self.aalto).min(KIVIA_ENINTAAN);
        for _ in 0..maara {
            // Kivet alkavat kentän laidalta.
            let reunalla = arpa.alle(LEVEYS as usize) as f32;
            let sivulla = arpa.alle(KORKEUS as usize) as f32;
            let mut paikka = if arpa.alle(2) == 0 {
                (reunalla, 0.0)
            } else {
                (0.0, sivulla)
            };
            if etaisyys(paikka, self.alus.paikka) < KIVEN_VALI_ALUKSESTA {
                paikka.0 = (paikka.0 + LEVEYS / 2.0).rem_euclid(LEVEYS);
            }
            let vauhti =
                arvo_vali(arpa, KIVEN_VAUHTI) as f32 + self.aalto as f32 * KIVEN_VAUHTI_AALLOSTA;
            self.lisaa_kivi(paikka, vauhti, Koko::Iso, arpa);
        }
    }

    fn lisaa_kivi(&mut self, paikka: Piste, vauhti: f32, koko: Koko, arpa: &mut Arpa) {
        let vauhti = vauhti.min(KIVEN_SUURIN_VAUHTI);
        self.kivet.push(Kivi {
            paikka,
            nopeus: suunnasta(arvo_kulma(arpa), vauhti),
            koko,
            kulma: arvo_kulma(arpa),
            pyorinta: (arpa.alle(21) as f32 - 10.0) / 10.0,
            muoto: arpa.alle(16),
        });
    }

    /// Ampuu aluksen nokasta. Palauttaa toden, jos laukaus lähti.
    pub fn ammu(&mut self) -> bool {
        let omia = self.ammukset.iter().filter(|a| a.oma).count();
        if self.tila != Tila::Lennossa || omia >= OMIA_AMMUKSIA {
            return false;
        }
        let nokka = suunnasta(self.alus.kulma, NOKAN_MATKA);
        let vauhti = suunnasta(self.alus.kulma, AMMUKSEN_VAUHTI);
        let mut paikka = self.alus.paikka;
        paikka.0 = (paikka.0 + nokka.0).rem_euclid(LEVEYS);
        paikka.1 = (paikka.1 + nokka.1).rem_euclid(KORKEUS);
        self.ammukset.push(Ammus {
            paikka,
            nopeus: (self.alus.nopeus.0 + vauhti.0, self.alus.nopeus.1 + vauhti.1),
            jaljella_ms: AMMUKSEN_IKA_MS,
            oma: true,
        });
        true
    }

    /// Hyppää satunnaiseen paikkaan. Paikka voi olla huono, siksi hyppy on hätäkeino.
    pub fn hyppaa(&mut self, arpa: &mut Arpa) -> bool {
        if self.tila != Tila::Lennossa || self.hyppy_ms > 0 {
            return false;
        }
        self.hyppy_ms = HYPYN_VALI_MS;
        self.alus.paikka = (
            arpa.alle(LEVEYS as usize) as f32,
            arpa.alle(KORKEUS as usize) as f32,
        );
        self.alus.nopeus = (0.0, 0.0);
        true
    }

    /// Etenee yhden `ASKEL_MS`:n verran.
    pub fn askel(&mut self, ohjaus: Ohjaus, arpa: &mut Arpa) -> Vec<Tapahtuma> {
        let mut tapahtumat = Vec::new();
        self.liikuta_kivia();
        if self.tila == Tila::Ohi {
            return tapahtumat;
        }
        self.ohjaa_alusta(ohjaus);
        self.liikuta_ammuksia();
        self.liikuta_lautasta(arpa, &mut tapahtumat);
        self.ammusten_osumat(arpa, &mut tapahtumat);
        self.aluksen_osumat(arpa, &mut tapahtumat);
        if self.tila == Tila::Ohi {
            return tapahtumat;
        }
        self.tuhon_odotus(&mut tapahtumat);
        if self.kivet.is_empty() {
            self.aallon_viive_ms = self.aallon_viive_ms.saturating_sub(ASKEL_MS);
            if self.aallon_viive_ms == 0 {
                self.aallon_viive_ms = AALLON_TAUKO_MS;
                self.aalto += 1;
                self.luo_aalto(arpa);
                tapahtumat.push(Tapahtuma::UusiAalto);
            }
        }
        tapahtumat
    }

    fn liikuta_kivia(&mut self) {
        for kivi in &mut self.kivet {
            liiku(&mut kivi.paikka, kivi.nopeus);
            kivi.kulma = (kivi.kulma + kivi.pyorinta * ASKEL_S).rem_euclid(TAU);
        }
    }

    fn ohjaa_alusta(&mut self, ohjaus: Ohjaus) {
        self.tyontaa = false;
        self.hyppy_ms = self.hyppy_ms.saturating_sub(ASKEL_MS);
        self.suoja_ms = self.suoja_ms.saturating_sub(ASKEL_MS);
        if self.tila != Tila::Lennossa {
            return;
        }
        let alus = &mut self.alus;
        let suurin = KAANTO * ASKEL_S;
        if ohjaus.kaanto != 0 {
            alus.kulma += ohjaus.kaanto as f32 * suurin;
        } else if let Some(tahtain) = ohjaus.tahtain {
            let (x, y) = (tahtain.0 - alus.paikka.0, tahtain.1 - alus.paikka.1);
            let ero = (y.atan2(x) - alus.kulma + PI).rem_euclid(TAU) - PI;
            alus.kulma += ero.clamp(-suurin, suurin);
        }
        alus.kulma = alus.kulma.rem_euclid(TAU);
        if ohjaus.tyonto {
            self.tyontaa = true;
            let lisays = suunnasta(alus.kulma, KIIHTYVYYS * ASKEL_S);
            alus.nopeus.0 += lisays.0;
            alus.nopeus.1 += lisays.1;
        }
        let jarru = 1.0 - KITKA * ASKEL_S;
        alus.nopeus = (alus.nopeus.0 * jarru, alus.nopeus.1 * jarru);
        let vauhti = alus.nopeus.0.hypot(alus.nopeus.1);
        if vauhti > SUURIN_VAUHTI {
            let kerroin = SUURIN_VAUHTI / vauhti;
            alus.nopeus = (alus.nopeus.0 * kerroin, alus.nopeus.1 * kerroin);
        }
        liiku(&mut alus.paikka, alus.nopeus);
    }

    fn liikuta_ammuksia(&mut self) {
        for ammus in &mut self.ammukset {
            liiku(&mut ammus.paikka, ammus.nopeus);
            ammus.jaljella_ms = ammus.jaljella_ms.saturating_sub(ASKEL_MS);
        }
        self.ammukset.retain(|ammus| ammus.jaljella_ms > 0);
    }

    fn liikuta_lautasta(&mut self, arpa: &mut Arpa, tapahtumat: &mut Vec<Tapahtuma>) {
        let Some(mut lautanen) = self.lautanen else {
            self.lautasen_viive_ms = self.lautasen_viive_ms.saturating_sub(ASKEL_MS);
            if self.lautasen_viive_ms == 0 {
                self.lautanen = Some(self.uusi_lautanen(arpa));
            }
            return;
        };
        // Lautanen ylittää kentän kerran ja poistuu vastakkaisesta laidasta.
        lautanen.paikka.0 += lautanen.nopeus.0 * ASKEL_S;
        lautanen.paikka.1 = (lautanen.paikka.1 + lautanen.nopeus.1 * ASKEL_S).rem_euclid(KORKEUS);
        lautanen.jaljella_ms = lautanen.jaljella_ms.saturating_sub(ASKEL_MS);
        if lautanen.jaljella_ms == 0 {
            self.lautanen = None;
            self.lautasen_viive_ms = lautasen_viive(self.aalto, arpa);
            return;
        }
        lautanen.kaantyy_ms = lautanen.kaantyy_ms.saturating_sub(ASKEL_MS);
        if lautanen.kaantyy_ms == 0 {
            lautanen.kaantyy_ms = LAUTASEN_KAANTO_MS;
            let vauhti = lautanen.nopeus.0.abs();
            lautanen.nopeus.1 = (arpa.alle(3) as f32 - 1.0) * vauhti * 0.6;
        }
        lautanen.laukaus_ms = lautanen.laukaus_ms.saturating_sub(ASKEL_MS);
        if lautanen.laukaus_ms == 0 {
            lautanen.laukaus_ms = LAUTASEN_LAUKAUS_MS;
            let kulma = if lautanen.pieni {
                let (x, y) = siirtyma(lautanen.paikka, self.alus.paikka);
                let hajonta = arpa.alle(TAHTAYKSEN_HAJONTA + 1) as f32;
                y.atan2(x) + (hajonta - TAHTAYKSEN_HAJONTA as f32 / 2.0) / 100.0
            } else {
                arvo_kulma(arpa)
            };
            self.ammukset.push(Ammus {
                paikka: lautanen.paikka,
                nopeus: suunnasta(kulma, LAUTASEN_AMMUKSEN_VAUHTI),
                jaljella_ms: LAUTASEN_AMMUKSEN_IKA_MS,
                oma: false,
            });
            tapahtumat.push(Tapahtuma::LautanenAmpui);
        }
        self.lautanen = Some(lautanen);
    }

    fn uusi_lautanen(&self, arpa: &mut Arpa) -> Lautanen {
        let pieni = self.pisteet >= PIENEN_LAUTASEN_RAJA;
        let vauhti = if pieni { 130.0 } else { 90.0 };
        let (x, suunta) = if arpa.alle(2) == 0 {
            (0.0, 1.0)
        } else {
            (LEVEYS, -1.0)
        };
        Lautanen {
            paikka: (x, arpa.alle(KORKEUS as usize) as f32),
            nopeus: (suunta * vauhti, 0.0),
            pieni,
            jaljella_ms: (LEVEYS / vauhti * 1000.0) as u32,
            kaantyy_ms: LAUTASEN_KAANTO_MS,
            laukaus_ms: LAUTASEN_LAUKAUS_MS,
        }
    }

    fn lisaa_pisteet(&mut self, maara: u32, tapahtumat: &mut Vec<Tapahtuma>) {
        self.pisteet = (self.pisteet + maara).min(SUURIN_PISTEMAARA);
        if self.pisteet >= self.seuraava_lisaelama {
            self.lisaelaman_vali += LISAELAMAN_KASVU;
            self.seuraava_lisaelama += self.lisaelaman_vali;
            self.elamat += 1;
            tapahtumat.push(Tapahtuma::Lisaelama);
        }
    }

    fn hajota_kivi(&mut self, i: usize, arpa: &mut Arpa, tapahtumat: &mut Vec<Tapahtuma>) {
        let kivi = self.kivet.swap_remove(i);
        self.lisaa_pisteet(kivi.koko.pisteet(), tapahtumat);
        tapahtumat.push(Tapahtuma::Kivi {
            paikka: kivi.paikka,
            koko: kivi.koko,
        });
        if let Some(sirpale) = kivi.koko.sirpale() {
            let vauhti = kivi.nopeus.0.hypot(kivi.nopeus.1) * SIRPALEEN_KIIHDYTYS;
            for _ in 0..2 {
                let lisa = arpa.alle(30) as f32;
                self.lisaa_kivi(kivi.paikka, vauhti + lisa, sirpale, arpa);
            }
        }
    }

    fn tuhoa_lautanen(&mut self, lautanen: Lautanen, arpa: &mut Arpa, t: &mut Vec<Tapahtuma>) {
        self.lautanen = None;
        self.lautasen_viive_ms = lautasen_viive(self.aalto, arpa);
        self.lisaa_pisteet(lautanen.pisteet(), t);
        t.push(Tapahtuma::LautanenTuhoutui {
            paikka: lautanen.paikka,
        });
    }

    fn ammusten_osumat(&mut self, arpa: &mut Arpa, tapahtumat: &mut Vec<Tapahtuma>) {
        let mut i = 0;
        while i < self.ammukset.len() {
            let ammus = self.ammukset[i];
            if ammus.oma && self.oma_ammus_osuu(ammus.paikka, arpa, tapahtumat) {
                self.ammukset.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn oma_ammus_osuu(
        &mut self,
        paikka: Piste,
        arpa: &mut Arpa,
        tapahtumat: &mut Vec<Tapahtuma>,
    ) -> bool {
        let kivi = self
            .kivet
            .iter()
            .position(|k| etaisyys(k.paikka, paikka) < k.koko.sade());
        if let Some(i) = kivi {
            self.hajota_kivi(i, arpa, tapahtumat);
            return true;
        }
        match self.lautanen {
            Some(lautanen) if etaisyys(lautanen.paikka, paikka) < lautanen.sade() => {
                self.tuhoa_lautanen(lautanen, arpa, tapahtumat);
                true
            }
            _ => false,
        }
    }

    fn aluksen_osumat(&mut self, arpa: &mut Arpa, tapahtumat: &mut Vec<Tapahtuma>) {
        if self.tila != Tila::Lennossa || self.suoja_ms > 0 {
            return;
        }
        let paikka = self.alus.paikka;
        let kivi = self
            .kivet
            .iter()
            .position(|k| etaisyys(k.paikka, paikka) < k.koko.sade() + ALUKSEN_SADE);
        if let Some(i) = kivi {
            self.hajota_kivi(i, arpa, tapahtumat);
            self.tuhoudu(tapahtumat);
            return;
        }
        if let Some(lautanen) = self.lautanen
            && etaisyys(lautanen.paikka, paikka) < lautanen.sade() + ALUKSEN_SADE
        {
            self.tuhoa_lautanen(lautanen, arpa, tapahtumat);
            self.tuhoudu(tapahtumat);
            return;
        }
        let ammus = self
            .ammukset
            .iter()
            .position(|a| !a.oma && etaisyys(a.paikka, paikka) < ALUKSEN_SADE);
        if let Some(i) = ammus {
            self.ammukset.swap_remove(i);
            self.tuhoudu(tapahtumat);
        }
    }

    fn tuhoudu(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        let paikka = self.alus.paikka;
        self.elamat -= 1;
        self.tyontaa = false;
        if self.elamat == 0 {
            self.tila = Tila::Ohi;
            self.ammukset.clear();
            self.lautanen = None;
            tapahtumat.push(Tapahtuma::Ohi { paikka });
        } else {
            self.tila = Tila::Tuho;
            self.odotus_ms = ODOTUS_MS;
            tapahtumat.push(Tapahtuma::Menetys { paikka });
        }
    }

    /// Uusi alus saa suojan, joten keskellä lentävä kivi ei tuhoa sitä heti.
    fn tuhon_odotus(&mut self, tapahtumat: &mut Vec<Tapahtuma>) {
        if self.tila != Tila::Tuho {
            return;
        }
        self.odotus_ms = self.odotus_ms.saturating_sub(ASKEL_MS);
        if self.odotus_ms == 0 {
            self.alus = Alus::keskella();
            self.suoja_ms = SUOJA_MS;
            self.tila = Tila::Lennossa;
            tapahtumat.push(Tapahtuma::Palasi);
        }
    }
}
