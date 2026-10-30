use klassikot8000::Arpa;
use klassikot8000::laivanupotus::{
    KOKO, Laivasto, Laukaustulos, Ruutu, SARAKKEET, Vaikeus, vastustaja,
};

use super::piirto;
use crate::yhteinen::aanet::Aani;
use crate::yhteinen::piirto::aanikytkimella;
use crate::yhteinen::syote::Syote;

// Kierroksen äänet tulevat peräkkäin, jotta laukaus ja sen seuraus erottuvat.
pub const TULOS_VIIVE_MS: u32 = 220;
const VASTAUS_VIIVE_MS: u32 = 950;
const LOPPU_VIIVE_MS: u32 = 900;
pub const TEHOSTE_KESTO_MS: u32 = 800;

/// Laukauksen lento ja osuman animaatio. Ajastus on sama kuin äänillä.
#[derive(Clone, Copy, Debug)]
pub struct Tehoste {
    /// Tosi, kun laukaus osuu pelaajan omalle laudalle.
    pub omalle: bool,
    pub ruutu: Ruutu,
    pub tulos: Laukaustulos,
    pub laukaus_ms: u32,
}

impl Tehoste {
    fn osuma_ms(&self) -> u32 {
        self.laukaus_ms + TULOS_VIIVE_MS
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vaihe {
    Alku,
    Ammunta,
    Voitto,
    Tappio,
}

pub struct Peli {
    pub vaihe: Vaihe,
    pub vaikeus: Vaikeus,
    pub oma: Laivasto,
    pub vastustaja: Laivasto,
    pub kursori: Ruutu,
    /// Edellisen kierroksen tulokset: ensin pelaajan, sitten vastustajan laukaus.
    pub viestit: [String; 2],
    pub ammuttu_jo: bool,
    /// Pyyntö palata käynnistimeen.
    pub lopeta: bool,
    pub aanet_paalla: bool,
    /// Soitettavat äänet ja niiden viiveet millisekunteina. `main.rs` tyhjentää jonon.
    pub aanijono: Vec<(Aani, u32)>,
    pub tehosteet: Vec<Tehoste>,
    /// Pelin alusta kulunut aika, josta kaikki animaatiot lasketaan.
    pub kello_ms: u32,
    arpa: Arpa,
}

impl Peli {
    pub fn new(siemen: u64) -> Self {
        let mut arpa = Arpa::new(siemen);
        Self {
            vaihe: Vaihe::Alku,
            vaikeus: Vaikeus::Normaali,
            oma: Laivasto::satunnainen(&mut arpa),
            vastustaja: Laivasto::satunnainen(&mut arpa),
            kursori: Ruutu { rivi: 0, sarake: 0 },
            viestit: Default::default(),
            ammuttu_jo: false,
            lopeta: false,
            aanet_paalla: true,
            aanijono: Vec::new(),
            tehosteet: Vec::new(),
            kello_ms: 0,
            arpa,
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        let kello = self.kello_ms;
        self.tehosteet
            .retain(|t| kello < t.osuma_ms() + TEHOSTE_KESTO_MS);
    }

    /// Onko ruutuun ammuttu laukaus vielä ilmassa? Tulos ei saa näkyä ennen osumaa.
    pub fn odottaa_osumaa(&self, omalle: bool, ruutu: Ruutu) -> bool {
        self.tehosteet
            .iter()
            .any(|t| t.omalle == omalle && t.ruutu == ruutu && self.kello_ms < t.osuma_ms())
    }

    /// Onko vastustajan laukaus vielä matkalla? Silloin sen tulosta ei kerrota.
    pub fn vastaus_kesken(&self) -> bool {
        self.tehosteet
            .iter()
            .any(|t| t.omalle && self.kello_ms < t.osuma_ms())
    }

    fn aloita_alusta(&mut self) {
        self.vaihe = Vaihe::Alku;
        self.oma = Laivasto::satunnainen(&mut self.arpa);
        self.vastustaja = Laivasto::satunnainen(&mut self.arpa);
        self.kursori = Ruutu { rivi: 0, sarake: 0 };
        self.viestit = Default::default();
        self.ammuttu_jo = false;
        self.tehosteet.clear();
    }

    /// Käsittelee syötteen ja kertoo, muuttuiko näkymä.
    pub fn kasittele(&mut self, syote: Syote) -> bool {
        if syote == Syote::Peru {
            self.lopeta = true;
            return false;
        }
        let kytkimella = matches!(syote, Syote::Klikkaus(x, y) if aanikytkimella(x, y));
        if syote == Syote::VaihdaAanet || kytkimella {
            self.aanet_paalla = !self.aanet_paalla;
            return true;
        }
        match self.vaihe {
            Vaihe::Alku => self.kasittele_alku(syote),
            Vaihe::Ammunta => self.kasittele_ammunta(syote),
            Vaihe::Voitto | Vaihe::Tappio => self.kasittele_loppu(syote),
        }
    }

    fn kasittele_alku(&mut self, syote: Syote) -> bool {
        let tasot = Vaikeus::KAIKKI;
        let valittu = match syote {
            Syote::Numero(numero) => (numero as usize)
                .checked_sub(1)
                .and_then(|i| tasot.get(i).copied()),
            Syote::Nuoli(_, muutos) => tasot.iter().position(|&t| t == self.vaikeus).map(|i| {
                tasot[i
                    .saturating_add_signed(muutos as isize)
                    .min(tasot.len() - 1)]
            }),
            Syote::Klikkaus(x, y) => piirto::vaikeus_pisteesta(x, y),
            _ => None,
        };
        if let Some(vaikeus) = valittu {
            let muuttui = vaikeus != self.vaikeus;
            self.vaikeus = vaikeus;
            return muuttui;
        }
        if matches!(syote, Syote::Ammu | Syote::Klikkaus(..)) {
            self.vaihe = Vaihe::Ammunta;
            self.aanijono.push((Aani::Ovi, 0));
            return true;
        }
        false
    }

    fn kasittele_ammunta(&mut self, syote: Syote) -> bool {
        let uusi = match syote {
            Syote::Hiiri(x, y) => match piirto::ruutu_pisteesta(x, y) {
                Some(ruutu) => ruutu,
                None => return false,
            },
            Syote::Klikkaus(x, y) => {
                let Some(ruutu) = piirto::ruutu_pisteesta(x, y) else {
                    return false;
                };
                self.kursori = ruutu;
                self.ammu(ruutu);
                return true;
            }
            Syote::Nuoli(drivi, dsarake) => Ruutu {
                rivi: siirra(self.kursori.rivi, drivi),
                sarake: siirra(self.kursori.sarake, dsarake),
            },
            Syote::Kirjain(kirjain) => match SARAKKEET.find(kirjain) {
                Some(sarake) => Ruutu {
                    sarake: sarake as u8,
                    ..self.kursori
                },
                None => return false,
            },
            Syote::Numero(numero) => Ruutu {
                rivi: (numero + 9) % 10,
                ..self.kursori
            },
            Syote::Ammu => {
                self.ammu(self.kursori);
                return true;
            }
            Syote::Peru | Syote::VaihdaAanet | Syote::Pyyhi => return false,
        };
        let muuttui = uusi != self.kursori || self.ammuttu_jo;
        self.kursori = uusi;
        self.ammuttu_jo = false;
        muuttui
    }

    fn kasittele_loppu(&mut self, syote: Syote) -> bool {
        let uudestaan = match syote {
            Syote::Kirjain('K') => true,
            Syote::Kirjain('E') => false,
            Syote::Klikkaus(x, y) => match piirto::nappi_pisteesta(x, y) {
                Some(piirto::Nappi::Uudestaan) => true,
                Some(piirto::Nappi::Lopeta) => false,
                None => return false,
            },
            _ => return false,
        };
        if uudestaan {
            self.aloita_alusta();
        } else {
            self.lopeta = true;
        }
        uudestaan
    }

    /// Yksi kierros: pelaaja ampuu, voittotarkistus, vastustaja ampuu, häviötarkistus.
    pub fn ammu(&mut self, ruutu: Ruutu) {
        let Some(tulos) = self.vastustaja.kasittele_laukaus(ruutu) else {
            self.ammuttu_jo = true;
            self.aanijono.push((Aani::Ahkaisy, 0));
            return;
        };
        self.ammuttu_jo = false;
        self.viestit = [tulos_teksti("Sinä ammuit", ruutu, tulos), String::new()];
        self.aanijono.push((Aani::Laukaus, 0));
        self.aanijono.push((tuloksen_aani(tulos), TULOS_VIIVE_MS));
        self.tehosteet.push(Tehoste {
            omalle: false,
            ruutu,
            tulos,
            laukaus_ms: self.kello_ms,
        });
        if self.vastustaja.kaikki_upotettu() {
            self.vaihe = Vaihe::Voitto;
            self.aanijono.push((Aani::Voitto, LOPPU_VIIVE_MS));
            return;
        }

        let laukaus = vastustaja::valitse(self.vaikeus, &self.oma.tiedot(), &mut self.arpa);
        let tulos = laukaus.and_then(|r| Some((r, self.oma.kasittele_laukaus(r)?)));
        if let Some((ruutu, tulos)) = tulos {
            self.viestit[1] = tulos_teksti("Vastustaja ampui", ruutu, tulos);
            self.aanijono.push((Aani::Tulipallo, VASTAUS_VIIVE_MS));
            self.aanijono
                .push((tuloksen_aani(tulos), VASTAUS_VIIVE_MS + TULOS_VIIVE_MS));
            self.tehosteet.push(Tehoste {
                omalle: true,
                ruutu,
                tulos,
                laukaus_ms: self.kello_ms + VASTAUS_VIIVE_MS,
            });
        }
        if self.oma.kaikki_upotettu() {
            self.vaihe = Vaihe::Tappio;
            self.aanijono
                .push((Aani::Tappio, VASTAUS_VIIVE_MS + LOPPU_VIIVE_MS));
        }
    }
}

fn tuloksen_aani(tulos: Laukaustulos) -> Aani {
    match tulos {
        Laukaustulos::Ohi => Aani::Roiske,
        Laukaustulos::Osuma => Aani::Rajahdys,
        Laukaustulos::Upposi(_) => Aani::Upotus,
    }
}

fn siirra(arvo: u8, muutos: i8) -> u8 {
    arvo.saturating_add_signed(muutos).min(KOKO as u8 - 1)
}

fn tulos_teksti(ampuja: &str, ruutu: Ruutu, tulos: Laukaustulos) -> String {
    match tulos {
        Laukaustulos::Ohi => format!("{ampuja} {ruutu}: ohi."),
        Laukaustulos::Osuma => format!("{ampuja} {ruutu}: OSUMA!"),
        Laukaustulos::Upposi(laiva) => format!("{ampuja} {ruutu}: OSUMA! {laiva} upposi!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yhteinen::piirto::aanikytkimen_keskipiste;

    fn pelissa(siemen: u64) -> Peli {
        let mut peli = Peli::new(siemen);
        assert!(peli.kasittele(Syote::Ammu));
        assert_eq!(peli.vaihe, Vaihe::Ammunta);
        peli
    }

    #[test]
    fn nappaimisto_siirtaa_kursoria_rajojen_sisalla() {
        let mut peli = pelissa(1);
        peli.kasittele(Syote::Nuoli(-1, -1));
        assert_eq!(peli.kursori, Ruutu { rivi: 0, sarake: 0 });
        peli.kasittele(Syote::Kirjain('J'));
        peli.kasittele(Syote::Numero(0));
        assert_eq!(peli.kursori.to_string(), "J10");
        peli.kasittele(Syote::Nuoli(1, 1));
        assert_eq!(peli.kursori.to_string(), "J10");
        peli.kasittele(Syote::Numero(3));
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.kursori.to_string(), "I3");
    }

    #[test]
    fn klikkaus_ampuu_ja_vastustaja_vastaa() {
        let mut peli = pelissa(1);
        let (x, y) = piirto::ruudun_keskipiste(Ruutu { rivi: 2, sarake: 3 });
        assert!(peli.kasittele(Syote::Klikkaus(x, y)));
        assert_eq!(peli.vastustaja.ammutut().maara(), 1);
        assert_eq!(peli.oma.ammutut().maara(), 1);
        assert!(peli.viestit[0].starts_with("Sinä ammuit D3: "));
        assert!(peli.viestit[1].starts_with("Vastustaja ampui "));
    }

    #[test]
    fn klikkaus_laudan_ulkopuolelle_ei_tee_mitaan() {
        let mut peli = pelissa(1);
        assert!(!peli.kasittele(Syote::Klikkaus(0, 0)));
        assert_eq!(peli.vastustaja.ammutut().maara(), 0);
    }

    #[test]
    fn sama_ruutu_antaa_huomautuksen_eika_kierrosta() {
        let mut peli = pelissa(1);
        peli.kasittele(Syote::Ammu);
        peli.kasittele(Syote::Ammu);
        assert!(peli.ammuttu_jo);
        assert_eq!(peli.oma.ammutut().maara(), 1);
    }

    #[test]
    fn voiton_jalkeen_vastustaja_ei_ammu_ja_tappio_tunnistetaan() {
        let mut voittoja = 0;
        let mut tappioita = 0;
        for siemen in 0..200 {
            let mut peli = pelissa(siemen);
            for i in 0..KOKO * KOKO {
                peli.ammu(Ruutu::new(i / KOKO, i % KOKO).unwrap());
                if peli.vaihe != Vaihe::Ammunta {
                    break;
                }
            }
            let omat = peli.oma.ammutut().maara();
            let vastustajan = peli.vastustaja.ammutut().maara();
            match peli.vaihe {
                Vaihe::Voitto => {
                    voittoja += 1;
                    assert!(peli.viestit[1].is_empty());
                    assert_eq!(omat + 1, vastustajan);
                }
                Vaihe::Tappio => {
                    tappioita += 1;
                    assert_eq!(omat, vastustajan);
                }
                _ => panic!("peli ei päättynyt"),
            }
        }
        assert!(voittoja > 0 && tappioita > 0);
    }

    #[test]
    fn lopussa_k_aloittaa_alusta_ja_e_lopettaa() {
        let mut peli = pelissa(1);
        peli.ammu(Ruutu { rivi: 0, sarake: 0 });
        peli.vaihe = Vaihe::Tappio;
        assert!(peli.kasittele(Syote::Kirjain('K')));
        assert_eq!(peli.vaihe, Vaihe::Alku);
        assert_eq!(peli.oma.ammutut().maara(), 0);
        assert!(peli.viestit[0].is_empty());

        let mut peli = pelissa(1);
        peli.vaihe = Vaihe::Voitto;
        peli.kasittele(Syote::Kirjain('E'));
        assert!(peli.lopeta);
    }

    #[test]
    fn lopussa_napit_toimivat_hiirella() {
        let mut peli = pelissa(1);
        peli.vaihe = Vaihe::Voitto;
        let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Uudestaan);
        assert!(peli.kasittele(Syote::Klikkaus(x, y)));
        assert_eq!(peli.vaihe, Vaihe::Alku);

        let mut peli = pelissa(1);
        peli.vaihe = Vaihe::Voitto;
        let (x, y) = piirto::napin_keskipiste(piirto::Nappi::Lopeta);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert!(peli.lopeta);
    }

    #[test]
    fn aanet_kytketaan_napilla_ja_kytkimella_kesken_pelin() {
        let mut peli = pelissa(1);
        assert!(peli.aanet_paalla);
        assert!(peli.kasittele(Syote::VaihdaAanet));
        assert!(!peli.aanet_paalla);
        let (x, y) = aanikytkimen_keskipiste();
        assert!(peli.kasittele(Syote::Klikkaus(x, y)));
        assert!(peli.aanet_paalla);
        assert_eq!(peli.vastustaja.ammutut().maara(), 0);
    }

    #[test]
    fn kytkimen_klikkaus_ei_aloita_pelia_ja_valinta_sailyy_uusinnassa() {
        let mut peli = Peli::new(1);
        let (x, y) = aanikytkimen_keskipiste();
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaihe, Vaihe::Alku);
        assert!(!peli.aanet_paalla);

        peli.kasittele(Syote::Ammu);
        peli.vaihe = Vaihe::Voitto;
        peli.kasittele(Syote::Kirjain('K'));
        assert!(!peli.aanet_paalla);
    }

    #[test]
    fn kierros_soittaa_laukauksen_ja_vastustajan_tulipallon() {
        let mut peli = pelissa(1);
        peli.aanijono.clear();
        peli.ammu(Ruutu { rivi: 0, sarake: 0 });
        assert_eq!(peli.aanijono[0], (Aani::Laukaus, 0));
        assert!(peli.aanijono.contains(&(Aani::Tulipallo, VASTAUS_VIIVE_MS)));

        peli.aanijono.clear();
        peli.ammu(Ruutu { rivi: 0, sarake: 0 });
        assert_eq!(peli.aanijono, [(Aani::Ahkaisy, 0)]);
    }

    #[test]
    fn tulos_paljastuu_vasta_osumahetkella_ja_tehoste_poistuu() {
        let mut peli = pelissa(1);
        let ruutu = Ruutu { rivi: 0, sarake: 0 };
        peli.ammu(ruutu);
        assert!(peli.odottaa_osumaa(false, ruutu));
        assert!(peli.vastaus_kesken());

        peli.edista(TULOS_VIIVE_MS);
        assert!(!peli.odottaa_osumaa(false, ruutu));
        assert!(peli.vastaus_kesken());

        peli.edista(VASTAUS_VIIVE_MS);
        assert!(!peli.vastaus_kesken());
        assert!(!peli.tehosteet.is_empty());

        peli.edista(TEHOSTE_KESTO_MS);
        assert!(peli.tehosteet.is_empty());
    }

    #[test]
    fn taso_valitaan_alkuruudulla_ja_sailyy_uusinnassa() {
        let mut peli = Peli::new(1);
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        peli.kasittele(Syote::Numero(3));
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        peli.kasittele(Syote::Nuoli(0, 1));
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        peli.kasittele(Syote::Nuoli(0, -1));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        peli.kasittele(Syote::Numero(0));
        peli.kasittele(Syote::Numero(4));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);

        let (x, y) = piirto::vaikeuden_keskipiste(Vaikeus::Helppo);
        peli.kasittele(Syote::Klikkaus(x, y));
        assert_eq!(peli.vaikeus, Vaikeus::Helppo);
        assert_eq!(peli.vaihe, Vaihe::Alku);

        peli.kasittele(Syote::Klikkaus(0, 0));
        assert_eq!(peli.vaihe, Vaihe::Ammunta);
        peli.vaihe = Vaihe::Voitto;
        peli.kasittele(Syote::Kirjain('K'));
        assert_eq!(peli.vaikeus, Vaikeus::Helppo);
    }

    #[test]
    fn numerot_eivat_vaihda_tasoa_kesken_pelin() {
        let mut peli = pelissa(1);
        peli.kasittele(Syote::Numero(1));
        assert_eq!(peli.vaikeus, Vaikeus::Normaali);
        assert_eq!(peli.kursori.rivi, 0);
    }

    #[test]
    fn esc_lopettaa_missa_tahansa_vaiheessa() {
        let mut peli = Peli::new(1);
        peli.kasittele(Syote::Peru);
        assert!(peli.lopeta);
    }
}
