use laivanupotus::Vaikeus;

use super::aanet::Aani;
use super::peli::{Peli, Syote};
use super::piirto;

pub struct Kortti {
    pub nimi: &'static str,
    pub kuvaus: &'static str,
    pub valmis: bool,
}

pub const PELIT: [Kortti; 2] = [
    Kortti {
        nimi: "LAIVANUPOTUS",
        kuvaus: "Upota vastustajan laivasto.",
        valmis: true,
    },
    Kortti {
        nimi: "2048",
        kuvaus: "Tulossa seuraavaksi.",
        valmis: false,
    },
];

/// Pelin valinta ja käynnissä oleva peli. Asetukset säilyvät, kun peli vaihtuu.
pub struct Kaynnistin {
    pub valittu: usize,
    pub peli: Option<Peli>,
    pub aanet_paalla: bool,
    pub lopeta: bool,
    /// Tosi, kun pelistä poistuttiin: sen viiveellä odottavat äänet eivät saa jäädä soimaan.
    pub hiljenna: bool,
    pub kello_ms: u32,
    aanijono: Vec<(Aani, u32)>,
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
            aanijono: Vec::new(),
            vaikeus: Vaikeus::Normaali,
            siemen,
        }
    }

    pub fn otsikko(&self) -> &'static str {
        if self.peli.is_some() {
            "Laivanupotus"
        } else {
            "Pelikone"
        }
    }

    pub fn kaynnista(&mut self) {
        let mut peli = Peli::new(self.siemen);
        // Sama siemen toistaisi saman laivaston, kun peli avataan uudelleen.
        self.siemen = self.siemen.wrapping_add(1);
        peli.vaikeus = self.vaikeus;
        peli.aanet_paalla = self.aanet_paalla;
        self.peli = Some(peli);
    }

    pub fn edista(&mut self, ms: u32) {
        self.kello_ms = self.kello_ms.saturating_add(ms);
        if let Some(peli) = &mut self.peli {
            peli.edista(ms);
        }
    }

    pub fn ota_aanet(&mut self) -> Vec<(Aani, u32)> {
        match &mut self.peli {
            Some(peli) => std::mem::take(&mut peli.aanijono),
            None => std::mem::take(&mut self.aanijono),
        }
    }

    pub fn kasittele(&mut self, syote: Syote) {
        if let Some(peli) = &mut self.peli {
            peli.kasittele(syote);
            self.aanet_paalla = peli.aanet_paalla;
            self.vaikeus = peli.vaikeus;
            if peli.lopeta {
                self.peli = None;
                self.hiljenna = true;
            }
            return;
        }
        let kytkimella = matches!(syote, Syote::Klikkaus(x, y) if piirto::aanikytkimella(x, y));
        if syote == Syote::VaihdaAanet || kytkimella {
            self.aanet_paalla = !self.aanet_paalla;
            return;
        }
        match syote {
            Syote::Peru => self.lopeta = true,
            Syote::Ammu => self.kaynnista(),
            Syote::Hiiri(x, y) => {
                if let Some(i) = piirto::kortti_pisteesta(x, y).filter(|&i| PELIT[i].valmis) {
                    self.valittu = i;
                }
            }
            Syote::Klikkaus(x, y) => {
                if let Some(i) = piirto::kortti_pisteesta(x, y)
                    && self.valitse(i)
                {
                    self.kaynnista();
                }
            }
            Syote::Numero(numero) => {
                if let Some(i) = (numero as usize)
                    .checked_sub(1)
                    .filter(|&i| i < PELIT.len())
                {
                    self.valitse(i);
                }
            }
            Syote::Nuoli(drivi, _) if drivi != 0 => self.siirra(drivi),
            _ => {}
        }
    }

    /// Keskeneräistä peliä ei voi valita, ähkäisy kertoo sen.
    fn valitse(&mut self, i: usize) -> bool {
        if PELIT[i].valmis {
            self.valittu = i;
        } else {
            self.aanijono.push((Aani::Ahkaisy, 0));
        }
        PELIT[i].valmis
    }

    /// Hyppää keskeneräisten pelien yli.
    fn siirra(&mut self, muutos: i8) {
        let mut i = self.valittu;
        while let Some(seuraava) = i
            .checked_add_signed(muutos as isize)
            .filter(|&j| j < PELIT.len())
        {
            i = seuraava;
            if PELIT[i].valmis {
                self.valittu = i;
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::peli::Vaihe;
    use laivanupotus::{KOKO, Ruutu};

    fn laivaruudut(peli: &Peli) -> Vec<bool> {
        (0..KOKO * KOKO)
            .map(|i| peli.oma.on_laiva(Ruutu::new(i / KOKO, i % KOKO).unwrap()))
            .collect()
    }

    #[test]
    fn valikko_kaynnistaa_pelin_ja_esc_palaa_valikkoon() {
        let mut k = Kaynnistin::new(1);
        assert!(k.peli.is_none());
        k.kasittele(Syote::Ammu);
        assert_eq!(k.peli.as_ref().map(|p| p.vaihe), Some(Vaihe::Alku));

        k.kasittele(Syote::Peru);
        assert!(k.peli.is_none());
        assert!(k.hiljenna);
        assert!(!k.lopeta);

        k.kasittele(Syote::Peru);
        assert!(k.lopeta);
    }

    #[test]
    fn kortin_klikkaus_kaynnistaa_ja_keskenerainen_ahkaisee() {
        let mut k = Kaynnistin::new(1);
        let (x, y) = piirto::kortin_keskipiste(1);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(k.peli.is_none());
        assert_eq!(k.valittu, 0);
        assert_eq!(k.ota_aanet(), [(Aani::Ahkaisy, 0)]);

        let (x, y) = piirto::kortin_keskipiste(0);
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(k.peli.is_some());
    }

    #[test]
    fn keskeneraista_ei_voi_valita_nuolilla_numerolla_eika_hiirella() {
        let mut k = Kaynnistin::new(1);
        k.kasittele(Syote::Nuoli(1, 0));
        k.kasittele(Syote::Nuoli(0, 1));
        k.kasittele(Syote::Numero(2));
        k.kasittele(Syote::Numero(0));
        k.kasittele(Syote::Numero(9));
        let (x, y) = piirto::kortin_keskipiste(1);
        k.kasittele(Syote::Hiiri(x, y));
        assert_eq!(k.valittu, 0);
        assert!(k.peli.is_none());
    }

    #[test]
    fn asetukset_sailyvat_kun_peli_vaihtuu_ja_laivasto_ei_toistu() {
        let mut k = Kaynnistin::new(1);
        let (x, y) = piirto::aanikytkimen_keskipiste();
        k.kasittele(Syote::Klikkaus(x, y));
        assert!(!k.aanet_paalla);
        assert!(k.peli.is_none());

        k.kasittele(Syote::Ammu);
        let ensimmainen = laivaruudut(k.peli.as_ref().unwrap());
        assert!(!k.peli.as_ref().unwrap().aanet_paalla);
        k.kasittele(Syote::Numero(3));
        k.kasittele(Syote::VaihdaAanet);
        k.kasittele(Syote::Peru);
        assert!(k.aanet_paalla);

        k.kasittele(Syote::Ammu);
        let peli = k.peli.as_ref().unwrap();
        assert_eq!(peli.vaikeus, Vaikeus::Vaikea);
        assert!(peli.aanet_paalla);
        assert_ne!(laivaruudut(peli), ensimmainen);
    }

    #[test]
    fn pelin_voitto_ja_e_palaavat_valikkoon() {
        let mut k = Kaynnistin::new(1);
        k.kaynnista();
        k.peli.as_mut().unwrap().vaihe = Vaihe::Voitto;
        k.kasittele(Syote::Kirjain('E'));
        assert!(k.peli.is_none());
        assert!(!k.lopeta);
    }

    #[test]
    fn nakymat_piirtyvat_kaatumatta() {
        let mut kuva = vec![0; piirto::LEVEYS * piirto::KORKEUS];
        let mut k = Kaynnistin::new(1);
        piirto::piirra_kaynnistin(&k, &mut kuva);
        k.kaynnista();
        k.edista(100);
        piirto::piirra_kaynnistin(&k, &mut kuva);
    }
}
