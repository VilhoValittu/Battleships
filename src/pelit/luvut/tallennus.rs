//! 2048-pelin tallennus tekstitiedostoon. Rikkinäinen tiedosto ei saa estää pelaamista,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kesken {
    pub ruudut: Ruudut,
    pub pisteet: u32,
    pub siirrot: u32,
    pub jatkettu: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tallennus {
    pub paras: u32,
    pub kesken: Option<Kesken>,
}

fn luku(teksti: &str) -> Option<u32> {
    teksti.parse().ok().filter(|&arvo| arvo <= SUURIN_LUKU)
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
        let mut teksti = format!("{OTSAKE}\nparas {}\n", self.paras);
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
        let mut jatkettu = false;
        for rivi in rivit {
            match rivi.split_once(' ') {
                Some(("paras", arvo)) => paras = luku(arvo),
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
        }
    }

    #[test]
    fn teksti_palautuu_samaksi() {
        let t = kokonainen();
        assert_eq!(Tallennus::tekstista(&t.tekstiksi()), t);
        let t = Tallennus {
            paras: 7,
            kesken: None,
        };
        assert_eq!(Tallennus::tekstista(&t.tekstiksi()), t);
    }

    #[test]
    fn vieras_tai_tyhja_tiedosto_antaa_tyhjan_tallennuksen() {
        assert_eq!(Tallennus::tekstista(""), Tallennus::default());
        assert_eq!(Tallennus::tekstista("paras 99\n"), Tallennus::default());
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
