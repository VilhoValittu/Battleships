//! 2048-pelin logiikka ilman ikkunaa, piirtoa tai syötettä.

use crate::Arpa;

pub const SIVU: usize = 4;
pub const TAVOITE: u32 = 2048;
const NELOSEN_KERROIN: usize = 10;

/// (rivi, sarake)
pub type Paikka = (usize, usize);
pub type Ruudut = [[u32; SIVU]; SIVU];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suunta {
    Ylos,
    Alas,
    Vasen,
    Oikea,
}

/// Yhden luvun matka siirrossa. `arvo` on luku ennen yhdistymistä.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Liike {
    pub mista: Paikka,
    pub mihin: Paikka,
    pub arvo: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Siirto {
    pub liikkeet: Vec<Liike>,
    pub yhdistyneet: Vec<Paikka>,
    pub pisteet: u32,
    pub uusi: Paikka,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lukulauta {
    ruudut: Ruudut,
    pub pisteet: u32,
}

/// Linjan askel 0 on reuna, jota kohti luvut liikkuvat.
fn paikka(suunta: Suunta, linja: usize, askel: usize) -> Paikka {
    match suunta {
        Suunta::Ylos => (askel, linja),
        Suunta::Alas => (SIVU - 1 - askel, linja),
        Suunta::Vasen => (linja, askel),
        Suunta::Oikea => (linja, SIVU - 1 - askel),
    }
}

impl Lukulauta {
    /// Uusi peli alkaa kahdella luvulla.
    pub fn new(arpa: &mut Arpa) -> Self {
        let mut lauta = Self::ruuduista([[0; SIVU]; SIVU]);
        lauta.lisaa_luku(arpa);
        lauta.lisaa_luku(arpa);
        lauta
    }

    pub fn ruuduista(ruudut: Ruudut) -> Self {
        Self { ruudut, pisteet: 0 }
    }

    pub fn ruudut(&self) -> Ruudut {
        self.ruudut
    }

    pub fn arvo(&self, (rivi, sarake): Paikka) -> u32 {
        self.ruudut[rivi][sarake]
    }

    pub fn suurin(&self) -> u32 {
        self.ruudut.iter().flatten().copied().max().unwrap_or(0)
    }

    pub fn on_siirtoja(&self) -> bool {
        (0..SIVU).any(|r| {
            (0..SIVU).any(|s| {
                let arvo = self.ruudut[r][s];
                arvo == 0
                    || (r + 1 < SIVU && self.ruudut[r + 1][s] == arvo)
                    || (s + 1 < SIVU && self.ruudut[r][s + 1] == arvo)
            })
        })
    }

    /// Työntää luvut reunaan ja lisää uuden luvun. `None`, jos mikään ei liikkunut.
    pub fn siirra(&mut self, suunta: Suunta, arpa: &mut Arpa) -> Option<Siirto> {
        let mut uudet = [[0; SIVU]; SIVU];
        let mut liikkeet = Vec::new();
        let mut yhdistyneet = Vec::new();
        let mut pisteet = 0;
        for linja in 0..SIVU {
            let mut kohde = 0;
            // Yhdistynyt luku ei saa yhdistyä samassa siirrossa uudelleen.
            let mut yhdistettava = None;
            for askel in 0..SIVU {
                let mista = paikka(suunta, linja, askel);
                let arvo = self.arvo(mista);
                if arvo == 0 {
                    continue;
                }
                let mihin = if yhdistettava == Some(arvo) {
                    yhdistettava = None;
                    let mihin = paikka(suunta, linja, kohde - 1);
                    yhdistyneet.push(mihin);
                    pisteet += arvo * 2;
                    mihin
                } else {
                    yhdistettava = Some(arvo);
                    kohde += 1;
                    paikka(suunta, linja, kohde - 1)
                };
                uudet[mihin.0][mihin.1] += arvo;
                liikkeet.push(Liike { mista, mihin, arvo });
            }
        }
        if uudet == self.ruudut {
            return None;
        }
        self.ruudut = uudet;
        self.pisteet += pisteet;
        let uusi = self.lisaa_luku(arpa)?;
        Some(Siirto {
            liikkeet,
            yhdistyneet,
            pisteet,
            uusi,
        })
    }

    fn lisaa_luku(&mut self, arpa: &mut Arpa) -> Option<Paikka> {
        let tyhjat: Vec<Paikka> = (0..SIVU)
            .flat_map(|r| (0..SIVU).map(move |s| (r, s)))
            .filter(|&p| self.arvo(p) == 0)
            .collect();
        if tyhjat.is_empty() {
            return None;
        }
        let paikka = tyhjat[arpa.alle(tyhjat.len())];
        let arvo = if arpa.alle(NELOSEN_KERROIN) == 0 {
            4
        } else {
            2
        };
        self.ruudut[paikka.0][paikka.1] = arvo;
        Some(paikka)
    }
}
