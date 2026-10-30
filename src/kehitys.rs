//! Kehityslippujen tuki. Mukana vain debug-käännöksessä, ei julkaistavassa exessä.

use std::fs::File;
use std::io::{self, Write};

use klassikot8000::laivanupotus::{Ruutu, Vaikeus};
use klassikot8000::luvut::{Ruudut, SIVU, Suunta};

use crate::kaynnistin::Laji;

pub struct Liput {
    pub siemen: Option<u64>,
    pub kuvakaappaus: Option<String>,
    pub siirrot: Vec<Ruutu>,
    pub vaikeus: Option<Vaikeus>,
    /// Ohittaa käynnistimen. Kunkin pelin omat liput tekevät saman.
    peli: Option<Laji>,
    /// 2048: luvut laudalla riveittäin, kuvakaappausta varten.
    pub lauta: Option<Ruudut>,
    /// 2048: siirrot kirjaimina U, D, L ja R.
    pub liu_ut: Vec<Suunta>,
    /// Tallennustiedosto. Ilman tätä kehitysajo ei lue eikä kirjoita tallennusta.
    pub tallennus: Option<String>,
    /// Kuvakaappauksen hetki viimeisen laukauksen jälkeen, animaatioiden tarkistukseen.
    pub aika_ms: u32,
}

impl Liput {
    pub fn laji(&self) -> Option<Laji> {
        if self.peli.is_some() {
            self.peli
        } else if self.vaikeus.is_some() || !self.siirrot.is_empty() {
            Some(Laji::Laivanupotus)
        } else if self.lauta.is_some() || !self.liu_ut.is_empty() {
            Some(Laji::Luvut)
        } else {
            None
        }
    }
}

// Riittää siihen, että kaikki tehosteet ovat päättyneet.
const OLETUSAIKA_MS: u32 = 3000;

pub fn lue_liput() -> Result<Liput, String> {
    let mut liput = Liput {
        siemen: None,
        kuvakaappaus: None,
        siirrot: Vec::new(),
        vaikeus: None,
        peli: None,
        lauta: None,
        liu_ut: Vec::new(),
        tallennus: None,
        aika_ms: OLETUSAIKA_MS,
    };
    let mut args = std::env::args().skip(1);
    while let Some(lippu) = args.next() {
        let arvo = args.next().ok_or(format!("{lippu}: arvo puuttuu"))?;
        match lippu.as_str() {
            "--seed" => liput.siemen = Some(arvo.parse().map_err(|_| "--seed: ei luku")?),
            "--screenshot" => liput.kuvakaappaus = Some(arvo),
            "--save" => liput.tallennus = Some(arvo),
            "--time" => liput.aika_ms = arvo.parse().map_err(|_| "--time: ei luku")?,
            "--game" => {
                liput.peli = Some(match arvo.as_str() {
                    "laivanupotus" => Laji::Laivanupotus,
                    "2048" => Laji::Luvut,
                    _ => return Err(format!("--game: {arvo}")),
                });
            }
            "--board" => {
                let luvut = arvo
                    .split(',')
                    .map(|osa| osa.parse::<u32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| format!("--board: {arvo}"))?;
                if luvut.len() != SIVU * SIVU {
                    return Err(format!("--board: tarvitaan {} lukua", SIVU * SIVU));
                }
                let mut ruudut = [[0; SIVU]; SIVU];
                for (i, luku) in luvut.into_iter().enumerate() {
                    ruudut[i / SIVU][i % SIVU] = luku;
                }
                liput.lauta = Some(ruudut);
            }
            "--slides" => {
                for kirjain in arvo.chars() {
                    liput.liu_ut.push(match kirjain {
                        'U' => Suunta::Ylos,
                        'D' => Suunta::Alas,
                        'L' => Suunta::Vasen,
                        'R' => Suunta::Oikea,
                        _ => return Err(format!("--slides: {kirjain}")),
                    });
                }
            }
            "--level" => {
                let taso = Vaikeus::KAIKKI
                    .into_iter()
                    .find(|taso| taso.nimi().eq_ignore_ascii_case(&arvo));
                liput.vaikeus = Some(taso.ok_or(format!("--level: {arvo}"))?);
            }
            "--moves" => {
                for osa in arvo.split(',') {
                    let ruutu = osa.parse().map_err(|_| format!("--moves: {osa}"))?;
                    liput.siirrot.push(ruutu);
                }
            }
            _ => return Err(format!("tuntematon lippu {lippu}")),
        }
    }
    Ok(liput)
}

/// Tallentaa kuvan pakkaamattomana PNG:nä, jotta pakkaaja ei tarvitse riippuvuutta.
pub fn tallenna_png(polku: &str, kuva: &[u32], leveys: usize, korkeus: usize) -> io::Result<()> {
    let mut raaka = Vec::with_capacity((leveys * 3 + 1) * korkeus);
    for rivi in kuva.chunks(leveys) {
        raaka.push(0);
        for &pikseli in rivi {
            raaka.extend_from_slice(&pikseli.to_be_bytes()[1..]);
        }
    }

    let mut zlib = vec![0x78, 0x01];
    let mut palat = raaka.chunks(0xFFFF).peekable();
    while let Some(pala) = palat.next() {
        zlib.push(palat.peek().is_none() as u8);
        zlib.extend_from_slice(&(pala.len() as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(pala.len() as u16)).to_le_bytes());
        zlib.extend_from_slice(pala);
    }
    zlib.extend_from_slice(&adler32(&raaka).to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(leveys as u32).to_be_bytes());
    ihdr.extend_from_slice(&(korkeus as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);

    let mut tiedosto = File::create(polku)?;
    tiedosto.write_all(b"\x89PNG\r\n\x1a\n")?;
    kirjoita_pala(&mut tiedosto, b"IHDR", &ihdr)?;
    kirjoita_pala(&mut tiedosto, b"IDAT", &zlib)?;
    kirjoita_pala(&mut tiedosto, b"IEND", &[])
}

fn kirjoita_pala(ulos: &mut impl Write, tyyppi: &[u8; 4], data: &[u8]) -> io::Result<()> {
    ulos.write_all(&(data.len() as u32).to_be_bytes())?;
    ulos.write_all(tyyppi)?;
    ulos.write_all(data)?;
    ulos.write_all(&crc32(tyyppi, data).to_be_bytes())
}

fn crc32(tyyppi: &[u8], data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &tavu in tyyppi.iter().chain(data) {
        crc ^= tavu as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &tavu in data {
        a = (a + tavu as u32) % 65521;
        b = (b + a) % 65521;
    }
    b << 16 | a
}
