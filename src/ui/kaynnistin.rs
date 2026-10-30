use klassikot8000::Vaikeus;

use super::aanet::Aani;
use super::luvut::Luvut;
use super::peli::{Peli, Syote};
use super::piirto;
use super::tallennus::Tallennus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laji {
    Laivanupotus,
    Luvut,
}

pub struct Kortti {
    pub laji: Laji,
    pub nimi: &'static str,
    pub kuvaus: &'static str,
}

pub const PELIT: [Kortti; 2] = [
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
];

pub enum Kaynnissa {
    Laivanupotus(Box<Peli>),
    Luvut(Box<Luvut>),
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
    /// 2048:n pysyvä tila. Käynnissä oleva peli siirtyy tänne `paivita_tallennus`-kutsulla.
    pub tallennus: Tallennus,
    /// Tosi, kun `main.rs`:n kuuluu kirjoittaa tallennus tiedostoon.
    pub tallenna: bool,
    vaikeus: Vaikeus,
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
            siemen,
        }
    }

    pub fn otsikko(&self) -> &'static str {
        match self.peli {
            Some(Kaynnissa::Laivanupotus(_)) => "Laivanupotus",
            Some(Kaynnissa::Luvut(_)) => "2048",
            None => "Pelikone",
        }
    }

    /// Laivanupotuksessa nuoli liikuttaa tähtäintä, mutta 2048:ssa toisto tekisi
    /// pohjassa pidetystä nuolesta siirtosarjan.
    pub fn nuolet_toistuvat(&self) -> bool {
        !matches!(self.peli, Some(Kaynnissa::Luvut(_)))
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
                if let Some(kesken) = self.tallennus.kesken {
                    peli.palauta(kesken);
                }
                peli.aanet_paalla = self.aanet_paalla;
                Kaynnissa::Luvut(Box::new(peli))
            }
        });
    }

    pub fn paivita_tallennus(&mut self) {
        if let Some(Kaynnissa::Luvut(peli)) = &self.peli {
            self.tallennus = Tallennus {
                paras: peli.paras,
                kesken: peli.kesken(),
            };
        }
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        match &mut self.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => peli.edista(ms),
            Some(Kaynnissa::Luvut(peli)) => peli.edista(ms),
            None => {}
        }
    }

    pub fn ota_aanet(&mut self) -> Vec<(Aani, u32)> {
        match &mut self.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => std::mem::take(&mut peli.aanijono),
            Some(Kaynnissa::Luvut(peli)) => std::mem::take(&mut peli.aanijono),
            None => Vec::new(),
        }
    }

    pub fn kasittele(&mut self, syote: Syote) {
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
        let kytkimella = matches!(syote, Syote::Klikkaus(x, y) if piirto::aanikytkimella(x, y));
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
    use crate::ui::luvut;
    use crate::ui::peli::Vaihe;
    use klassikot8000::lukulauta::Lukulauta;
    use klassikot8000::{KOKO, Ruutu};

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
        assert_eq!(k.valittu, 1);
        k.kasittele(Syote::Numero(0));
        k.kasittele(Syote::Numero(9));
        assert_eq!(k.valittu, 1);

        let (x, y) = piirto::kortin_keskipiste(0);
        k.kasittele(Syote::Hiiri(x, y));
        assert_eq!(k.valittu, 0);
        let (x, y) = piirto::kortin_keskipiste(1);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(matches!(k.peli, Some(Kaynnissa::Luvut(_))));
    }

    #[test]
    fn asetukset_sailyvat_kun_peli_vaihtuu_ja_laivasto_ei_toistu() {
        let mut k = Kaynnistin::new(1);
        let (x, y) = piirto::aanikytkimen_keskipiste();
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
        let mut kuva = vec![0; piirto::LEVEYS * piirto::KORKEUS];
        let mut k = Kaynnistin::new(1);
        piirto::piirra_kaynnistin(&k, &mut kuva);
        for laji in [Laji::Laivanupotus, Laji::Luvut] {
            k.kaynnista(laji);
            k.edista(100);
            piirto::piirra_kaynnistin(&k, &mut kuva);
        }
    }
}
