//! 2048-pelin tila, parhaat tulokset ja ääniasetus tekstitiedostoon. Rikkinäinen tiedosto ei saa estää pelaamista,
//! joten kelvottomat tiedot jätetään huomiotta. Tiedostoa käsittelee vain `main.rs`.

use std::fs;
use std::io;
use std::path::Path;
#[cfg(not(debug_assertions))]
use std::path::PathBuf;

use klassikot8000::luvut::{Lukulauta, Ruudut, SIVU};

const OTSAKE: &str = "klassikot8000 1";
// Rajat pitävät muokatun tiedoston luvut sellaisina, ettei pistelasku voi ylivuotaa.
const SUURIN_LAATTA: u32 = 1 << 20;
const SUURIN_LUKU: u32 = 1_000_000_000;
pub const PARHAITA: usize = 3;
pub const NIMEN_PITUUS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kesken {
    pub ruudut: Ruudut,
    pub pisteet: u32,
    pub siirrot: u32,
    pub jatkettu: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tulos {
    pub nimi: String,
    pub pisteet: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tallennus {
    pub paras: u32,
    pub kesken: Option<Kesken>,
    /// Suurin tulos ensin, enintään `PARHAITA` kappaletta.
    pub parhaat: Vec<Tulos>,
    /// Oletus on epätosi, joten vanhassa tallennuksessa äänet ovat päällä.
    pub hiljaa: bool,
}

fn luku(teksti: &str) -> Option<u32> {
    teksti.parse().ok().filter(|&arvo| arvo <= SUURIN_LUKU)
}

/// Tasapisteissä vanhempi tulos säilyttää paikkansa, joten sen ohi ei pääse.
pub fn mahtuu_parhaisiin(parhaat: &[Tulos], pisteet: u32) -> bool {
    pisteet > 0 && (parhaat.len() < PARHAITA || parhaat.iter().any(|t| t.pisteet < pisteet))
}

pub fn lisaa_tulos(parhaat: &mut Vec<Tulos>, tulos: Tulos) {
    let paikka = parhaat.partition_point(|t| t.pisteet >= tulos.pisteet);
    parhaat.insert(paikka, tulos);
    parhaat.truncate(PARHAITA);
}

fn tulos(teksti: &str) -> Option<Tulos> {
    let (pisteet, nimi) = teksti.split_once(' ')?;
    let pituus = nimi.chars().count();
    let kelpaa = (1..=NIMEN_PITUUS).contains(&pituus)
        && nimi.chars().all(|m| m.is_uppercase() || m.is_ascii_digit());
    let pisteet = luku(pisteet).filter(|&p| p > 0)?;
    kelpaa.then(|| Tulos {
        nimi: nimi.to_string(),
        pisteet,
    })
}

fn ruudut(teksti: &str) -> Option<Ruudut> {
    let mut ruudut = [[0; SIVU]; SIVU];
    let mut osat = teksti.split(',');
    for arvo in ruudut.iter_mut().flatten() {
        *arvo = osat.next()?.parse().ok()?;
        if *arvo != 0 && (*arvo < 2 || *arvo > SUURIN_LAATTA || !arvo.is_power_of_two()) {
            return None;
        }
    }
    osat.next().is_none().then_some(ruudut)
}

impl Tallennus {
    pub fn tekstiksi(&self) -> String {
        let mut teksti = format!(
            "{OTSAKE}\nparas {}\nhiljaa {}\n",
            self.paras, self.hiljaa as u8,
        );
        for tulos in &self.parhaat {
            teksti += &format!("top {} {}\n", tulos.pisteet, tulos.nimi);
        }
        if let Some(kesken) = &self.kesken {
            let luvut: Vec<String> = kesken.ruudut.iter().flatten().map(u32::to_string).collect();
            teksti += &format!(
                "lauta {}\npisteet {}\nsiirrot {}\njatkettu {}\n",
                luvut.join(","),
                kesken.pisteet,
                kesken.siirrot,
                kesken.jatkettu as u8,
            );
        }
        teksti
    }

    pub fn tekstista(teksti: &str) -> Self {
        let mut rivit = teksti.lines();
        if rivit.next() != Some(OTSAKE) {
            return Self::default();
        }
        let (mut paras, mut lauta, mut pisteet, mut siirrot) = (None, None, None, None);
        let (mut jatkettu, mut hiljaa) = (false, false);
        let mut parhaat = Vec::new();
        for rivi in rivit {
            match rivi.split_once(' ') {
                Some(("paras", arvo)) => paras = luku(arvo),
                Some(("hiljaa", arvo)) => hiljaa = arvo == "1",
                Some(("top", arvo)) => {
                    if let Some(tulos) = tulos(arvo) {
                        lisaa_tulos(&mut parhaat, tulos);
                    }
                }
                Some(("lauta", arvo)) => lauta = ruudut(arvo),
                Some(("pisteet", arvo)) => pisteet = luku(arvo),
                Some(("siirrot", arvo)) => siirrot = luku(arvo),
                Some(("jatkettu", arvo)) => jatkettu = arvo == "1",
                _ => {}
            }
        }
        let kesken = match (lauta, pisteet, siirrot) {
            (Some(ruudut), Some(pisteet), Some(siirrot))
                if Lukulauta::ruuduista(ruudut).on_siirtoja() =>
            {
                Some(Kesken {
                    ruudut,
                    pisteet,
                    siirrot,
                    jatkettu,
                })
            }
            _ => None,
        };
        Self {
            paras: paras.unwrap_or(0),
            kesken,
            parhaat,
            hiljaa,
        }
    }
}

#[cfg(not(debug_assertions))]
pub fn oletuspolku() -> Option<PathBuf> {
    let kansio = PathBuf::from(std::env::var_os("APPDATA")?);
    Some(kansio.join("klassikot8000").join("tallennus.txt"))
}

pub fn lue_tiedosto(polku: &Path) -> Tallennus {
    fs::read_to_string(polku).map_or_else(|_| Tallennus::default(), |t| Tallennus::tekstista(&t))
}

/// Kirjoittaa ensin väliaikaiseen tiedostoon, jottei keskeytynyt kirjoitus tuhoa tallennusta.
pub fn kirjoita_tiedosto(polku: &Path, tallennus: &Tallennus) -> io::Result<()> {
    if let Some(kansio) = polku.parent() {
        fs::create_dir_all(kansio)?;
    }
    let valiaikainen = polku.with_extension("tmp");
    fs::write(&valiaikainen, tallennus.tekstiksi())?;
    fs::rename(&valiaikainen, polku)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KESKEN: Kesken = Kesken {
        ruudut: [[2, 4, 8, 16], [0; 4], [0, 1024, 0, 0], [0; 4]],
        pisteet: 1234,
        siirrot: 56,
        jatkettu: true,
    };

    fn kokonainen() -> Tallennus {
        Tallennus {
            paras: 5000,
            kesken: Some(KESKEN),
            parhaat: vec![
                Tulos {
                    nimi: "VÄINÖ".to_string(),
                    pisteet: 5000,
                },
                Tulos {
                    nimi: "A1".to_string(),
                    pisteet: 300,
                },
            ],
            hiljaa: true,
        }
    }

    #[test]
    fn teksti_palautuu_samaksi() {
        let t = kokonainen();
        assert_eq!(Tallennus::tekstista(&t.tekstiksi()), t);
        let t = Tallennus {
            paras: 7,
            kesken: None,
            parhaat: Vec::new(),
            hiljaa: false,
        };
        assert_eq!(Tallennus::tekstista(&t.tekstiksi()), t);
    }

    #[test]
    fn vieras_tai_tyhja_tiedosto_antaa_tyhjan_tallennuksen() {
        assert_eq!(Tallennus::tekstista(""), Tallennus::default());
        assert_eq!(Tallennus::tekstista("paras 99\n"), Tallennus::default());
    }

    #[test]
    fn parhaat_pysyvat_jarjestyksessa_ja_tasapisteilla_vanhempi_sailyy() {
        let tulos = |nimi: &str, pisteet| Tulos {
            nimi: nimi.to_string(),
            pisteet,
        };
        let mut parhaat = Vec::new();
        assert!(!mahtuu_parhaisiin(&parhaat, 0));
        for (nimi, pisteet) in [("B", 200), ("C", 100), ("A", 300)] {
            assert!(mahtuu_parhaisiin(&parhaat, pisteet));
            lisaa_tulos(&mut parhaat, tulos(nimi, pisteet));
        }
        assert!(!mahtuu_parhaisiin(&parhaat, 100));
        assert!(mahtuu_parhaisiin(&parhaat, 101));
        lisaa_tulos(&mut parhaat, tulos("D", 200));
        assert_eq!(parhaat, [tulos("A", 300), tulos("B", 200), tulos("D", 200)]);
    }

    #[test]
    fn kelvoton_tulos_hylataan_ja_muokattu_lista_jarjestetaan() {
        let t = Tallennus::tekstista(
            "klassikot8000 1\ntop 5 PIENI\ntop 9 ISO\ntop 0 NOLLA\ntop x KISSA\n\
             top 7 pieni\ntop 7 LIIANPITKA\ntop 7\ntop 7 KAKSI OSAA\n",
        );
        let pisteet: Vec<u32> = t.parhaat.iter().map(|t| t.pisteet).collect();
        assert_eq!(pisteet, [9, 5]);
        assert_eq!(t.parhaat[0].nimi, "ISO");
    }

    #[test]
    fn vanhassa_tallennuksessa_aanet_ovat_paalla() {
        let t = Tallennus::tekstista("klassikot8000 1\nparas 7\n");
        assert_eq!(t.paras, 7);
        assert!(!t.hiljaa);
        assert!(t.parhaat.is_empty());
    }

    #[test]
    fn kelvoton_lauta_hylataan_mutta_paras_sailyy() {
        let hyva = kokonainen().tekstiksi();
        let lauta = "2,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0";
        for huono in [
            "3,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0",
            "1,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0",
            "4294967296,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0",
            "2147483648,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0",
            "2,4,8",
            "2,4,8,16,0,0,0,0,0,1024,0,0,0,0,0,0,0",
            "kissa",
            // täysi lauta ilman siirtoja
            "2,4,2,4,4,2,4,2,2,4,2,4,4,2,4,2",
        ] {
            let t = Tallennus::tekstista(&hyva.replace(lauta, huono));
            assert_eq!(t.paras, 5000, "{huono}");
            assert_eq!(t.kesken, None, "{huono}");
        }
        let t = Tallennus::tekstista(&hyva.replace("paras 5000", "paras -1"));
        assert_eq!(t.paras, 0);
        assert!(t.kesken.is_some());
    }

    #[test]
    fn tiedosto_luodaan_ja_luetaan_ja_puuttuva_on_tyhja() {
        let kansio = std::env::temp_dir().join(format!("k8000_{}", std::process::id()));
        let polku = kansio.join("ala").join("tallennus.txt");
        assert_eq!(lue_tiedosto(&polku), Tallennus::default());
        kirjoita_tiedosto(&polku, &kokonainen()).unwrap();
        kirjoita_tiedosto(&polku, &kokonainen()).unwrap();
        assert_eq!(lue_tiedosto(&polku), kokonainen());
        fs::remove_dir_all(&kansio).unwrap();
    }
}
