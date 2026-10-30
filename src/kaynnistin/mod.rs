//! Pelin valinta ja käynnissä olevan pelin hallinta. Uusi peli lisätään tänne
//! korttina (`PELIT`) ja `Kaynnissa`-muunnelmana.

pub mod piirto;

use klassikot8000::laivanupotus::Vaikeus;
use klassikot8000::pasianssi;

use crate::pelit::laivanupotus::peli::Peli;
use crate::pelit::luvut::peli::Luvut;
use crate::pelit::luvut::tallennus::Tallennus;
use crate::pelit::pasianssi::peli::{self as pasianssi_peli, Pasianssi};
use crate::pelit::tiilirikko::peli::Tiilirikko;
use crate::yhteinen::aanet::Aani;
use crate::yhteinen::piirto::{aanikytkimella, ruudulta_kuvaan};
use crate::yhteinen::syote::Syote;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laji {
    Laivanupotus,
    Luvut,
    Pasianssi,
    Tiilirikko,
}

pub struct Kortti {
    pub laji: Laji,
    pub nimi: &'static str,
    pub kuvaus: &'static str,
}

pub const PELIT: [Kortti; 4] = [
    Kortti {
        laji: Laji::Laivanupotus,
        nimi: "LAIVANUPOTUS",
        kuvaus: "Upota vastustajan laivasto.",
    },
    Kortti {
        laji: Laji::Luvut,
        nimi: "2048",
        kuvaus: "Yhdistä luvut ja saavuta 2048.",
    },
    Kortti {
        laji: Laji::Pasianssi,
        nimi: "PASIANSSI",
        kuvaus: "Klassinen pasianssi, kolme tasoa.",
    },
    Kortti {
        laji: Laji::Tiilirikko,
        nimi: "TIILIRIKKO",
        kuvaus: "Riko tiilet, pidä pallo kentällä.",
    },
];

pub enum Kaynnissa {
    Laivanupotus(Box<Peli>),
    Luvut(Box<Luvut>),
    Pasianssi(Box<Pasianssi>),
    Tiilirikko(Box<Tiilirikko>),
}

/// Pelin valinta ja käynnissä oleva peli. Asetukset säilyvät, kun peli vaihtuu.
pub struct Kaynnistin {
    pub valittu: usize,
    pub peli: Option<Kaynnissa>,
    pub aanet_paalla: bool,
    pub lopeta: bool,
    /// Tosi, kun pelistä poistuttiin: sen viiveellä odottavat äänet eivät saa jäädä soimaan.
    pub hiljenna: bool,
    pub kello_ms: u32,
    /// 2048:n pysyvä tila, parhaat tulokset ja ääniasetus. Nykytila siirtyy tänne `paivita_tallennus`-kutsulla.
    pub tallennus: Tallennus,
    /// Tosi, kun `main.rs`:n kuuluu kirjoittaa tallennus tiedostoon.
    pub tallenna: bool,
    vaikeus: Vaikeus,
    pasianssin_vaikeus: pasianssi::Vaikeus,
    siemen: u64,
}

impl Kaynnistin {
    pub fn new(siemen: u64) -> Self {
        Self {
            valittu: 0,
            peli: None,
            aanet_paalla: true,
            lopeta: false,
            hiljenna: false,
            kello_ms: 0,
            tallennus: Tallennus::default(),
            tallenna: false,
            vaikeus: Vaikeus::Normaali,
            pasianssin_vaikeus: pasianssi::Vaikeus::Normaali,
            siemen,
        }
    }

    pub fn otsikko(&self) -> &'static str {
        match self.peli {
            Some(Kaynnissa::Laivanupotus(_)) => "Laivanupotus",
            Some(Kaynnissa::Luvut(_)) => "2048",
            Some(Kaynnissa::Pasianssi(_)) => "Pasianssi",
            Some(Kaynnissa::Tiilirikko(_)) => "Tiilirikko",
            None => "Pelikone",
        }
    }

    /// Reaaliaikainen peli tarvitsee tiheämmän päivityksen kuin vuoropelit.
    pub fn toimintapeli(&self) -> bool {
        matches!(self.peli, Some(Kaynnissa::Tiilirikko(_)))
    }

    /// Pohjassa olevat nuolet. Näppäintoisto olisi mailan ohjaukseen liian nykivä.
    pub fn aseta_ohjaus(&mut self, vasen: bool, oikea: bool) {
        if let Some(Kaynnissa::Tiilirikko(peli)) = &mut self.peli {
            peli.ohjaus = oikea as i8 - vasen as i8;
        }
    }

    /// Ikkuna ei ole aktiivinen, joten reaaliaikainen peli jää odottamaan.
    pub fn keskeyta(&mut self) {
        if let Some(Kaynnissa::Tiilirikko(peli)) = &mut self.peli {
            peli.keskeyta();
        }
    }

    /// Laivanupotuksessa nuoli liikuttaa tähtäintä, mutta 2048:ssa toisto tekisi
    /// pohjassa pidetystä nuolesta siirtosarjan.
    pub fn nuolet_toistuvat(&self) -> bool {
        !matches!(self.peli, Some(Kaynnissa::Luvut(_)))
    }

    /// Muuttaa ruudun pikselin sen näkymän koordinaateiksi, joka on nyt esillä.
    pub fn nakyman_piste(&self, x: i32, y: i32) -> (i32, i32) {
        match &self.peli {
            Some(Kaynnissa::Pasianssi(peli)) if peli.vaihe != pasianssi_peli::Vaihe::Alku => (x, y),
            _ => ruudulta_kuvaan(x, y),
        }
    }

    pub fn kaynnista(&mut self, laji: Laji) {
        let siemen = self.siemen;
        // Sama siemen toistaisi saman alkutilanteen, kun peli avataan uudelleen.
        self.siemen = self.siemen.wrapping_add(1);
        self.peli = Some(match laji {
            Laji::Laivanupotus => {
                let mut peli = Peli::new(siemen);
                peli.vaikeus = self.vaikeus;
                peli.aanet_paalla = self.aanet_paalla;
                Kaynnissa::Laivanupotus(Box::new(peli))
            }
            Laji::Luvut => {
                let mut peli = Luvut::new(siemen);
                peli.paras = self.tallennus.paras;
                peli.parhaat = self.tallennus.parhaat.clone();
                if let Some(kesken) = self.tallennus.kesken {
                    peli.palauta(kesken);
                }
                peli.aanet_paalla = self.aanet_paalla;
                Kaynnissa::Luvut(Box::new(peli))
            }
            Laji::Pasianssi => {
                let mut peli = Pasianssi::new(siemen);
                peli.vaikeus = self.pasianssin_vaikeus;
                peli.aanet_paalla = self.aanet_paalla;
                Kaynnissa::Pasianssi(Box::new(peli))
            }
            Laji::Tiilirikko => {
                let mut peli = Tiilirikko::new(siemen);
                peli.paras = self.tallennus.tiilirikko;
                peli.parhaat = self.tallennus.tiilirikon_parhaat.clone();
                peli.aanet_paalla = self.aanet_paalla;
                Kaynnissa::Tiilirikko(Box::new(peli))
            }
        });
    }

    /// Ottaa käyttöön tiedostosta luetun tallennuksen.
    pub fn lataa(&mut self, tallennus: Tallennus) {
        self.aanet_paalla = !tallennus.hiljaa;
        self.tallennus = tallennus;
    }

    pub fn paivita_tallennus(&mut self) {
        self.tallennus.hiljaa = !self.aanet_paalla;
        match &self.peli {
            Some(Kaynnissa::Luvut(peli)) => {
                self.tallennus.paras = peli.paras;
                self.tallennus.parhaat = peli.parhaat.clone();
                self.tallennus.kesken = peli.kesken();
            }
            Some(Kaynnissa::Tiilirikko(peli)) => {
                self.tallennus.tiilirikko = peli.paras;
                self.tallennus.tiilirikon_parhaat = peli.parhaat.clone();
            }
            _ => {}
        }
    }

    /// Ikkuna suljetaan: nimeä odottava tulos kirjataan, ettei se katoa.
    pub fn viimeistele(&mut self) {
        match &mut self.peli {
            Some(Kaynnissa::Luvut(peli)) => peli.vahvista_nimi(),
            Some(Kaynnissa::Tiilirikko(peli)) => peli.vahvista_nimi(),
            _ => {}
        }
        self.paivita_tallennus();
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        match &mut self.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => peli.edista(ms),
            Some(Kaynnissa::Luvut(peli)) => peli.edista(ms),
            Some(Kaynnissa::Pasianssi(peli)) => peli.edista(ms),
            Some(Kaynnissa::Tiilirikko(peli)) => {
                peli.edista(ms);
                // Tiilirikko päättyy ajan kuluessa eikä syötteeseen, joten ennätys kirjataan täällä.
                if std::mem::take(&mut peli.uusi_tulos) {
                    self.paivita_tallennus();
                    self.tallenna = true;
                }
            }
            None => {}
        }
    }

    pub fn ota_aanet(&mut self) -> Vec<(Aani, u32)> {
        match &mut self.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Luvut(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Pasianssi(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Tiilirikko(peli)) => std::mem::take(&mut peli.aanijono),
            None => Vec::new(),
        }
    }

    pub fn kasittele(&mut self, syote: Syote) {
        let aanet_ennen = self.aanet_paalla;
        self.kasittele_nakyma(syote);
        let uusi_tulos = match &mut self.peli {
            Some(Kaynnissa::Luvut(peli)) => std::mem::take(&mut peli.uusi_tulos),
            Some(Kaynnissa::Tiilirikko(peli)) => std::mem::take(&mut peli.uusi_tulos),
            _ => false,
        };
        if uusi_tulos || self.aanet_paalla != aanet_ennen {
            self.paivita_tallennus();
            self.tallenna = true;
        }
    }

    fn kasittele_nakyma(&mut self, syote: Syote) {
        let poistuu = match &mut self.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                self.vaikeus = peli.vaikeus;
                peli.lopeta
            }
            Some(Kaynnissa::Luvut(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            Some(Kaynnissa::Pasianssi(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                self.pasianssin_vaikeus = peli.vaikeus;
                peli.lopeta
            }
            Some(Kaynnissa::Tiilirikko(peli)) => {
                peli.kasittele(syote);
                self.aanet_paalla = peli.aanet_paalla;
                peli.lopeta
            }
            None => {
                self.kasittele_valikko(syote);
                return;
            }
        };
        if poistuu {
            self.paivita_tallennus();
            self.tallenna = true;
            self.peli = None;
            self.hiljenna = true;
        }
    }

    fn kasittele_valikko(&mut self, syote: Syote) {
        let kytkimella = matches!(syote, Syote::Klikkaus(x, y) if aanikytkimella(x, y));
        if syote == Syote::VaihdaAanet || kytkimella {
            self.aanet_paalla = !self.aanet_paalla;
            return;
        }
        match syote {
            Syote::Peru => self.lopeta = true,
            Syote::Ammu => self.kaynnista(PELIT[self.valittu].laji),
            Syote::Hiiri(x, y) => {
                if let Some(i) = piirto::kortti_pisteesta(x, y) {
                    self.valittu = i;
                }
            }
            Syote::Klikkaus(x, y) => {
                if let Some(i) = piirto::kortti_pisteesta(x, y) {
                    self.valittu = i;
                    self.kaynnista(PELIT[i].laji);
                }
            }
            Syote::Numero(numero) => {
                if let Some(i) = (numero as usize)
                    .checked_sub(1)
                    .filter(|&i| i < PELIT.len())
                {
                    self.valittu = i;
                }
            }
            Syote::Nuoli(drivi, _) => {
                self.valittu = self
                    .valittu
                    .saturating_add_signed(drivi as isize)
                    .min(PELIT.len() - 1);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pelit::laivanupotus::peli::Vaihe;
    use crate::pelit::luvut::peli as luvut;
    use crate::yhteinen::piirto::{
        KORKEUS, LEVEYS, RUUDUN_KORKEUS, RUUDUN_LEVEYS, aanikytkimen_keskipiste,
    };
    use klassikot8000::laivanupotus::{KOKO, Ruutu};
    use klassikot8000::luvut::Lukulauta;

    fn laivaruudut(peli: &Peli) -> Vec<bool> {
        (0..KOKO * KOKO)
            .map(|i| peli.oma.on_laiva(Ruutu::new(i / KOKO, i % KOKO).unwrap()))
            .collect()
    }

    fn laivanupotus(k: &Kaynnistin) -> &Peli {
        match &k.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => peli,
            _ => panic!("laivanupotus ei ole käynnissä"),
        }
    }

    fn luvut(k: &mut Kaynnistin) -> &mut Luvut {
        match &mut k.peli {
            Some(Kaynnissa::Luvut(peli)) => peli,
            _ => panic!("2048 ei ole käynnissä"),
        }
    }

    #[test]
    fn valikko_kaynnistaa_pelin_ja_esc_palaa_valikkoon() {
        let mut k = Kaynnistin::new(1);
        assert!(k.peli.is_none());
        k.kasittele(Syote::Ammu);
        assert_eq!(laivanupotus(&k).vaihe, Vaihe::Alku);
        assert_eq!(k.otsikko(), "Laivanupotus");

        k.kasittele(Syote::Peru);
        assert!(k.peli.is_none());
        assert!(k.hiljenna);
        assert!(!k.lopeta);

        k.kasittele(Syote::Peru);
        assert!(k.lopeta);
    }

    #[test]
    fn jokainen_peli_valitaan_ja_kaynnistetaan_korttinsa_kautta() {
        let mut k = Kaynnistin::new(1);
        k.kasittele(Syote::Numero(2));
        assert_eq!(k.valittu, 1);
        k.kasittele(Syote::Ammu);
        assert_eq!(luvut(&mut k).vaihe, luvut::Vaihe::Alku);
        assert_eq!(k.otsikko(), "2048");
        k.kasittele(Syote::Peru);

        k.kasittele(Syote::Nuoli(-1, 0));
        k.kasittele(Syote::Nuoli(-1, 0));
        assert_eq!(k.valittu, 0);
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(1, 0));
        assert_eq!(k.valittu, 3);
        k.kasittele(Syote::Numero(0));
        k.kasittele(Syote::Numero(9));
        assert_eq!(k.valittu, 3);

        let (x, y) = piirto::kortin_keskipiste(0);
        k.kasittele(Syote::Hiiri(x, y));
        assert_eq!(k.valittu, 0);
        let (x, y) = piirto::kortin_keskipiste(1);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Luvut(_))));
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(2);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Pasianssi(_))));
        assert_eq!(k.otsikko(), "Pasianssi");
        k.kasittele(Syote::Peru);

        let (x, y) = piirto::kortin_keskipiste(3);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Tiilirikko(_))));
        assert_eq!(k.otsikko(), "Tiilirikko");
        assert!(k.toimintapeli());
    }

    #[test]
    fn tiilirikon_ennatys_tallentuu_pelin_paattyessa_ja_palautuu() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Tiilirikko);
        k.kasittele(Syote::Ammu);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Tiilirikko(peli)) = &mut k.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        peli.paras = 70;
        peli.lauta.elamat = 1;
        peli.lauta.pallo = (300.0, 600.0);
        peli.lauta.nopeus = (0.0, 300.0);
        k.edista(5);
        assert!(k.tallenna);
        assert_eq!(k.tallennus.tiilirikko, 70);

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        uusi.kaynnista(Laji::Tiilirikko);
        let Some(Kaynnissa::Tiilirikko(peli)) = &uusi.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.paras, 70);
    }

    #[test]
    fn tiilirikon_top3_tallentuu_nimen_jalkeen_ja_sulkiessa_ja_palautuu() {
        let paattyva = |k: &mut Kaynnistin| {
            k.kaynnista(Laji::Tiilirikko);
            k.kasittele(Syote::Ammu);
            k.kasittele(Syote::Ammu);
            let Some(Kaynnissa::Tiilirikko(peli)) = &mut k.peli else {
                panic!("tiilirikko ei ole käynnissä");
            };
            peli.lauta.pisteet = 70;
            peli.lauta.elamat = 1;
            peli.lauta.pallo = (300.0, 600.0);
            peli.lauta.nopeus = (0.0, 300.0);
            k.edista(5);
        };

        let mut k = Kaynnistin::new(1);
        paattyva(&mut k);
        k.tallenna = false;
        k.kasittele(Syote::Kirjain('A'));
        k.kasittele(Syote::Ammu);
        assert!(k.tallenna);
        assert_eq!(k.tallennus.tiilirikon_parhaat[0].nimi, "A");

        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        uusi.kaynnista(Laji::Tiilirikko);
        let Some(Kaynnissa::Tiilirikko(peli)) = &uusi.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.parhaat[0].pisteet, 70);

        let mut k = Kaynnistin::new(1);
        paattyva(&mut k);
        k.viimeistele();
        assert_eq!(k.tallennus.tiilirikon_parhaat[0].nimi, "NIMETÖN");
    }

    #[test]
    fn nuolet_ohjaavat_tiilirikkoa_pohjassa_ollessaan() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Tiilirikko);
        k.aseta_ohjaus(true, false);
        let Some(Kaynnissa::Tiilirikko(peli)) = &k.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.ohjaus, -1);
        k.aseta_ohjaus(true, true);
        let Some(Kaynnissa::Tiilirikko(peli)) = &k.peli else {
            panic!("tiilirikko ei ole käynnissä");
        };
        assert_eq!(peli.ohjaus, 0);
    }

    #[test]
    fn pasianssin_taso_sailyy_ja_hiiri_muuntuu_nakyman_mukaan() {
        let mut k = Kaynnistin::new(1);
        assert_eq!(k.nakyman_piste(285, 0), (0, 0));
        k.kaynnista(Laji::Pasianssi);
        assert_eq!(k.nakyman_piste(285, 0), (0, 0));
        k.kasittele(Syote::Numero(3));
        k.kasittele(Syote::Ammu);
        assert_eq!(k.nakyman_piste(285, 7), (285, 7));
        k.kasittele(Syote::Peru);

        k.kaynnista(Laji::Pasianssi);
        k.kasittele(Syote::Ammu);
        let Some(Kaynnissa::Pasianssi(peli)) = &k.peli else {
            panic!("pasianssi ei ole käynnissä");
        };
        assert_eq!(peli.lauta.vaikeus, pasianssi::Vaikeus::Vaikea);
    }

    #[test]
    fn asetukset_sailyvat_kun_peli_vaihtuu_ja_laivasto_ei_toistu() {
        let mut k = Kaynnistin::new(1);
        let (x, y) = aanikytkimen_keskipiste();
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(!k.aanet_paalla);
        assert!(k.peli.is_none());

        k.kasittele(Syote::Ammu);
        let ensimmainen = laivaruudut(laivanupotus(&k));
        assert!(!laivanupotus(&k).aanet_paalla);
        k.kasittele(Syote::Numero(3));
        k.kasittele(Syote::VaihdaAanet);
        k.kasittele(Syote::Peru);
        assert!(k.aanet_paalla);

        k.kasittele(Syote::Ammu);
        let peli = laivanupotus(&k);
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        assert!(peli.aanet_paalla);
        assert_ne!(laivaruudut(peli), ensimmainen);
    }

    #[test]
    fn paras_tulos_ja_aanivalinta_sailyvat_2048_pelin_yli() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Ammu);
        luvut(&mut k).lauta = Lukulauta::ruuduista([[4, 4, 0, 0], [0; 4], [0; 4], [0; 4]]);
        k.kasittele(Syote::Nuoli(0, -1));
        k.kasittele(Syote::VaihdaAanet);
        assert!(!k.nuolet_toistuvat());
        k.kasittele(Syote::Peru);
        assert!(k.nuolet_toistuvat());

        k.kaynnista(Laji::Luvut);
        assert_eq!(luvut(&mut k).paras, 8);
        assert!(!luvut(&mut k).aanet_paalla);
    }

    #[test]
    fn keskenerainen_2048_palautuu_valikon_kautta_ja_tiedoston_tilasta() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Ammu);
        luvut(&mut k).lauta = Lukulauta::ruuduista([[4, 4, 0, 0], [0; 4], [0; 4], [0; 4]]);
        k.kasittele(Syote::Nuoli(0, -1));
        let lauta = luvut(&mut k).lauta.clone();
        k.kasittele(Syote::Peru);
        assert!(k.tallenna);

        // sama kuin ohjelman uudelleenkäynnistys: tila kulkee tekstin kautta
        let mut uusi = Kaynnistin::new(2);
        uusi.tallennus = Tallennus::tekstista(&k.tallennus.tekstiksi());
        uusi.kaynnista(Laji::Luvut);
        assert!(luvut(&mut uusi).jatkuu);
        uusi.kasittele(Syote::Ammu);
        assert_eq!(luvut(&mut uusi).lauta, lauta);
        assert_eq!(luvut(&mut uusi).paras, 8);

        // ikkunan sulkeminen kesken pelin: main.rs päivittää ennen kirjoitusta
        uusi.kasittele(Syote::Nuoli(1, 0));
        uusi.paivita_tallennus();
        assert_eq!(uusi.tallennus.kesken.unwrap().siirrot, 2);
    }

    #[test]
    fn aanivalinta_tallentuu_heti_ja_palautuu_tiedoston_tilasta() {
        let mut k = Kaynnistin::new(1);
        k.kasittele(Syote::VaihdaAanet);
        assert!(k.tallenna);
        assert!(k.tallennus.hiljaa);

        // sama kuin ohjelman uudelleenkäynnistys: tila kulkee tekstin kautta
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        assert!(!uusi.aanet_paalla);
        uusi.kaynnista(Laji::Laivanupotus);
        assert!(!laivanupotus(&uusi).aanet_paalla);

        // pelin sisällä vaihdettu valinta tallentuu sekin
        uusi.kasittele(Syote::VaihdaAanet);
        assert!(uusi.tallenna);
        assert!(!uusi.tallennus.hiljaa);
    }

    #[test]
    fn top3_tallentuu_heti_ja_palautuu_tiedoston_tilasta() {
        let lukkoon = [
            [2, 2, 16, 8],
            [16, 32, 64, 16],
            [4, 16, 8, 32],
            [16, 32, 64, 128],
        ];
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Ammu);
        luvut(&mut k).lauta = Lukulauta::ruuduista(lukkoon);
        k.kasittele(Syote::Nuoli(0, -1));
        k.kasittele(Syote::Kirjain('Ö'));
        assert!(!k.tallenna);
        k.kasittele(Syote::Ammu);
        assert!(k.tallenna);

        // sama kuin ohjelman uudelleenkäynnistys: tila kulkee tekstin kautta
        let mut uusi = Kaynnistin::new(2);
        uusi.lataa(Tallennus::tekstista(&k.tallennus.tekstiksi()));
        uusi.kaynnista(Laji::Luvut);
        assert_eq!(luvut(&mut uusi).parhaat[0].nimi, "Ö");
        assert_eq!(luvut(&mut uusi).parhaat[0].pisteet, 4);

        // ikkunan sulkeminen nimeä kirjoittaessa ei hävitä tulosta
        uusi.kasittele(Syote::Ammu);
        luvut(&mut uusi).lauta = Lukulauta::ruuduista(lukkoon);
        uusi.kasittele(Syote::Nuoli(0, -1));
        uusi.viimeistele();
        assert_eq!(uusi.tallennus.parhaat.len(), 2);
    }

    #[test]
    fn aloittamaton_uusi_peli_ei_kumoa_parasta() {
        let mut k = Kaynnistin::new(1);
        k.tallennus.paras = 900;
        k.kaynnista(Laji::Luvut);
        k.kasittele(Syote::Peru);
        assert_eq!(k.tallennus.paras, 900);
        assert_eq!(k.tallennus.kesken, None);
    }

    #[test]
    fn pelin_lopun_e_palaa_valikkoon() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista(Laji::Laivanupotus);
        if let Some(Kaynnissa::Laivanupotus(peli)) = &mut k.peli {
            peli.vaihe = Vaihe::Voitto;
        }
        k.kasittele(Syote::Kirjain('E'));
        assert!(k.peli.is_none());
        assert!(!k.lopeta);
    }

    #[test]
    fn nakymat_piirtyvat_kaatumatta() {
        let mut ruutu = vec![0; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
        let mut kuva = vec![0; LEVEYS * KORKEUS];
        let mut k = Kaynnistin::new(1);
        piirto::piirra(&k, &mut ruutu, &mut kuva);
        for laji in [
            Laji::Laivanupotus,
            Laji::Luvut,
            Laji::Pasianssi,
            Laji::Tiilirikko,
        ] {
            k.kaynnista(laji);
            k.edista(100);
            piirto::piirra(&k, &mut ruutu, &mut kuva);
            k.kasittele(Syote::Ammu);
            piirto::piirra(&k, &mut ruutu, &mut kuva);
        }
    }
}
