//! Palikka8000: kolme samaa suoraan. Vaihto purkautuu heti loppuun asti, ja kierrokset
//! jäävät talteen, jotta näkymä voi toistaa ne yksi kerrallaan.

use crate::Arpa;

pub const SIVU: usize = 8;
pub const VAREJA: usize = 8;
/// Sama raja kuin nettilistan muissa loputtomissa peleissä.
pub const SUURIN_TULOS: u32 = 999_999;
const RUUDUN_PISTEET: u32 = 10;
const TASON_PISTEET: u32 = 1000;

/// (rivi, sarake)
pub type Paikka = (usize, usize);
pub type Ruudut = [[Option<Palikka>; SIVU]; SIVU];
type Poistot = [[bool; SIVU]; SIVU];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palikka {
    Tavallinen(u8),
    /// Syntyy neljän ryhmästä ja räjäyttää poistuessaan ympäröivät ruudut.
    Pommi(u8),
    /// Syntyy viiden suorasta. Vaihto naapurin kanssa poistaa kaikki naapurin väriset.
    Kaikki,
}

impl Palikka {
    pub fn vari(self) -> Option<u8> {
        match self {
            Palikka::Tavallinen(vari) | Palikka::Pommi(vari) => Some(vari),
            Palikka::Kaikki => None,
        }
    }
}

/// Palikka putoaa rivilta `mista_rivi` paikkaan `mihin`. Uudet tulevat laudan yläpuolelta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pudotus {
    pub mista_rivi: i32,
    pub mihin: Paikka,
    pub palikka: Palikka,
}

/// Yksi poisto ja sen jälkeinen putoaminen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kierros {
    /// Lauta ennen poistoa.
    pub ennen: Ruudut,
    pub poistuneet: Vec<Paikka>,
    /// Lauta poiston jälkeen ennen putoamista. Uudet erikoispalikat ovat jo paikoillaan.
    pub valissa: Ruudut,
    pub luodut: Vec<Paikka>,
    pub rajahdykset: Vec<Paikka>,
    pub pudotukset: Vec<Pudotus>,
    pub pisteet: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vaihto {
    pub a: Paikka,
    pub b: Paikka,
    pub kierrokset: Vec<Kierros>,
    pub pisteet: u32,
    pub uusi_taso: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palikkalauta {
    ruudut: Ruudut,
    pub pisteet: u32,
    pub taso: u32,
    /// Nykyisellä tasolla kertyneet pisteet.
    pub tason_edistys: u32,
    /// Siirtoja ei enää ole.
    pub ohi: bool,
}

/// Montako väriä tasolla on. Lisäväri harventaa suoria, joten peli lopulta jumiutuu.
pub fn varien_maara(taso: u32) -> usize {
    match taso {
        0..=2 => 6,
        3..=5 => 7,
        _ => VAREJA,
    }
}

/// Tason läpäisyyn tarvittavat pisteet. Tason kerroin nostaa myös ansaintaa, joten taso
/// kestää suunnilleen yhtä monta siirtoa.
pub fn tason_tavoite(taso: u32) -> u32 {
    TASON_PISTEET * taso
}

pub fn vierekkain(a: Paikka, b: Paikka) -> bool {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1) == 1
}

fn laudalla(p: Paikka) -> bool {
    p.0 < SIVU && p.1 < SIVU
}

fn kaikki_paikat() -> impl Iterator<Item = Paikka> {
    (0..SIVU).flat_map(|rivi| (0..SIVU).map(move |sarake| (rivi, sarake)))
}

fn vari_kohdassa(ruudut: &Ruudut, (rivi, sarake): Paikka) -> Option<u8> {
    ruudut[rivi][sarake].and_then(Palikka::vari)
}

fn vaihda_ruudut(ruudut: &mut Ruudut, a: Paikka, b: Paikka) {
    let vanha = ruudut[a.0][a.1];
    ruudut[a.0][a.1] = ruudut[b.0][b.1];
    ruudut[b.0][b.1] = vanha;
}

/// Pisin samanvärinen suora, joka kulkee paikan kautta.
fn suora_kohdassa(ruudut: &Ruudut, p: Paikka) -> usize {
    let Some(vari) = vari_kohdassa(ruudut, p) else {
        return 0;
    };
    let pituus = |muutos: (i32, i32)| {
        let (mut r, mut s) = (p.0 as i32 + muutos.0, p.1 as i32 + muutos.1);
        let mut maara = 0;
        while (0..SIVU as i32).contains(&r)
            && (0..SIVU as i32).contains(&s)
            && vari_kohdassa(ruudut, (r as usize, s as usize)) == Some(vari)
        {
            maara += 1;
            r += muutos.0;
            s += muutos.1;
        }
        maara
    };
    (1 + pituus((0, -1)) + pituus((0, 1))).max(1 + pituus((-1, 0)) + pituus((1, 0)))
}

struct Ryhma {
    ruudut: Vec<Paikka>,
    pisin: usize,
    vari: u8,
}

impl Ryhma {
    fn erikois(&self) -> Option<Palikka> {
        if self.pisin >= 5 {
            Some(Palikka::Kaikki)
        } else if self.ruudut.len() >= 4 {
            Some(Palikka::Pommi(self.vari))
        } else {
            None
        }
    }
}

/// Kaikki vähintään kolmen suorat. Toisiinsa koskevat samanväriset suorat ovat yksi ryhmä.
fn etsi_ryhmat(ruudut: &Ruudut) -> Vec<Ryhma> {
    let mut suorassa = [[0; SIVU]; SIVU];
    for p in kaikki_paikat() {
        let pituus = suora_kohdassa(ruudut, p);
        if pituus >= 3 {
            suorassa[p.0][p.1] = pituus;
        }
    }
    let mut kayty = [[false; SIVU]; SIVU];
    let mut ryhmat = Vec::new();
    for alku in kaikki_paikat() {
        if suorassa[alku.0][alku.1] == 0 || kayty[alku.0][alku.1] {
            continue;
        }
        let Some(vari) = vari_kohdassa(ruudut, alku) else {
            continue;
        };
        let mut ryhma = Ryhma {
            ruudut: Vec::new(),
            pisin: 0,
            vari,
        };
        kayty[alku.0][alku.1] = true;
        let mut jono = vec![alku];
        while let Some(p) = jono.pop() {
            ryhma.ruudut.push(p);
            ryhma.pisin = ryhma.pisin.max(suorassa[p.0][p.1]);
            let naapurit = [
                (p.0.wrapping_sub(1), p.1),
                (p.0 + 1, p.1),
                (p.0, p.1.wrapping_sub(1)),
                (p.0, p.1 + 1),
            ];
            for n in naapurit {
                if laudalla(n)
                    && !kayty[n.0][n.1]
                    && suorassa[n.0][n.1] > 0
                    && vari_kohdassa(ruudut, n) == Some(vari)
                {
                    kayty[n.0][n.1] = true;
                    jono.push(n);
                }
            }
        }
        ryhmat.push(ryhma);
    }
    ryhmat
}

/// Ensimmäinen siirto, joka tuottaa suoran tai laukaisee kaikkipalikan.
fn loyda_siirto(ruudut: &Ruudut) -> Option<(Paikka, Paikka)> {
    let mut kopio = *ruudut;
    for a in kaikki_paikat() {
        for b in [(a.0, a.1 + 1), (a.0 + 1, a.1)] {
            if !laudalla(b) {
                continue;
            }
            let kaikkipalikka = [a, b]
                .iter()
                .any(|p| ruudut[p.0][p.1] == Some(Palikka::Kaikki));
            vaihda_ruudut(&mut kopio, a, b);
            let suora = suora_kohdassa(&kopio, a) >= 3 || suora_kohdassa(&kopio, b) >= 3;
            vaihda_ruudut(&mut kopio, a, b);
            if kaikkipalikka || suora {
                return Some((a, b));
            }
        }
    }
    None
}

impl Palikkalauta {
    /// Alkulauta ilman valmiita suoria, ja siinä on aina ainakin yksi siirto.
    pub fn new(arpa: &mut Arpa) -> Self {
        let maara = varien_maara(1);
        loop {
            let mut ruudut: Ruudut = [[None; SIVU]; SIVU];
            for (rivi, sarake) in kaikki_paikat() {
                let vari = loop {
                    let vari = arpa.alle(maara) as u8;
                    let vaaka = sarake >= 2
                        && vari_kohdassa(&ruudut, (rivi, sarake - 1)) == Some(vari)
                        && vari_kohdassa(&ruudut, (rivi, sarake - 2)) == Some(vari);
                    let pysty = rivi >= 2
                        && vari_kohdassa(&ruudut, (rivi - 1, sarake)) == Some(vari)
                        && vari_kohdassa(&ruudut, (rivi - 2, sarake)) == Some(vari);
                    if !vaaka && !pysty {
                        break vari;
                    }
                };
                ruudut[rivi][sarake] = Some(Palikka::Tavallinen(vari));
            }
            if loyda_siirto(&ruudut).is_some() {
                return Self::ruuduista(ruudut);
            }
        }
    }

    pub fn ruuduista(ruudut: Ruudut) -> Self {
        Self {
            ruudut,
            pisteet: 0,
            taso: 1,
            tason_edistys: 0,
            ohi: false,
        }
    }

    pub fn ruudut(&self) -> &Ruudut {
        &self.ruudut
    }

    pub fn palikka(&self, (rivi, sarake): Paikka) -> Option<Palikka> {
        self.ruudut[rivi][sarake]
    }

    pub fn on_siirtoja(&self) -> bool {
        self.vihje().is_some()
    }

    pub fn vihje(&self) -> Option<(Paikka, Paikka)> {
        loyda_siirto(&self.ruudut)
    }

    /// Vaihtaa naapuripalikat. Jos vaihto ei tuota mitään, lauta pysyy ennallaan ja tulos on `None`.
    pub fn vaihda(&mut self, a: Paikka, b: Paikka, arpa: &mut Arpa) -> Option<Vaihto> {
        if !laudalla(a) || !laudalla(b) || !vierekkain(a, b) {
            return None;
        }
        vaihda_ruudut(&mut self.ruudut, a, b);
        let (poistuu, mut laukeamatta) = self.kaikkipalikan_poistot(a, b);
        if poistuu.is_none() && etsi_ryhmat(&self.ruudut).is_empty() {
            vaihda_ruudut(&mut self.ruudut, a, b);
            return None;
        }

        let mut kierrokset = Vec::new();
        let mut alku = poistuu;
        let vaihdetut = [a, b];
        let mut vaihdetut = vaihdetut.as_slice();
        loop {
            let ketju = kierrokset.len() as u32 + 1;
            let Some(kierros) = self.kierros(alku.take(), vaihdetut, &laukeamatta, ketju, arpa)
            else {
                break;
            };
            kierrokset.push(kierros);
            vaihdetut = &[];
            // Sama paikka voi myöhemmin saada uuden kaikkipalikan.
            laukeamatta.clear();
        }

        let pisteet: u32 = kierrokset.iter().map(|k: &Kierros| k.pisteet).sum();
        self.pisteet = (self.pisteet + pisteet).min(SUURIN_TULOS);
        self.tason_edistys += pisteet;
        let mut uusi_taso = false;
        while self.tason_edistys >= tason_tavoite(self.taso) {
            self.tason_edistys -= tason_tavoite(self.taso);
            self.taso += 1;
            uusi_taso = true;
        }
        self.ohi = !self.on_siirtoja();
        Some(Vaihto {
            a,
            b,
            kierrokset,
            pisteet,
            uusi_taso,
        })
    }

    /// Vaihdettu kaikkipalikka poistuu naapurin värin kanssa. Kaksi yhdessä tyhjentää laudan.
    /// Palauttaa myös paikat, joissa olevat kaikkipalikat on jo käytetty.
    fn kaikkipalikan_poistot(&self, a: Paikka, b: Paikka) -> (Option<Poistot>, Vec<Paikka>) {
        let (pa, pb) = (self.palikka(a), self.palikka(b));
        let mut poistuu = [[false; SIVU]; SIVU];
        match (pa, pb) {
            (Some(Palikka::Kaikki), Some(Palikka::Kaikki)) => {
                return (Some([[true; SIVU]; SIVU]), vec![a, b]);
            }
            (Some(Palikka::Kaikki), Some(toinen)) | (Some(toinen), Some(Palikka::Kaikki)) => {
                for p in kaikki_paikat() {
                    poistuu[p.0][p.1] = self.palikka(p).and_then(Palikka::vari) == toinen.vari();
                }
            }
            _ => return (None, Vec::new()),
        }
        let kaytetty = if pa == Some(Palikka::Kaikki) { a } else { b };
        poistuu[kaytetty.0][kaytetty.1] = true;
        (Some(poistuu), vec![kaytetty])
    }

    /// Poistaa ryhmät ja antaa muiden pudota. `None`, kun poistettavaa ei enää ole.
    fn kierros(
        &mut self,
        alku: Option<Poistot>,
        vaihdetut: &[Paikka],
        laukeamatta: &[Paikka],
        ketju: u32,
        arpa: &mut Arpa,
    ) -> Option<Kierros> {
        let ryhmat = etsi_ryhmat(&self.ruudut);
        let mut poistuu = alku.unwrap_or([[false; SIVU]; SIVU]);
        for ryhma in &ryhmat {
            for p in &ryhma.ruudut {
                poistuu[p.0][p.1] = true;
            }
        }
        let rajahdykset = self.laajenna_poistot(&mut poistuu, laukeamatta, arpa);
        let poistuneet: Vec<Paikka> = kaikki_paikat().filter(|p| poistuu[p.0][p.1]).collect();
        if poistuneet.is_empty() {
            return None;
        }

        let ennen = self.ruudut;
        for p in &poistuneet {
            self.ruudut[p.0][p.1] = None;
        }
        let mut luodut = Vec::new();
        for ryhma in &ryhmat {
            if let Some(erikois) = ryhma.erikois() {
                let paikka = vaihdetut
                    .iter()
                    .find(|p| ryhma.ruudut.contains(p))
                    .copied()
                    .unwrap_or(ryhma.ruudut[ryhma.ruudut.len() / 2]);
                self.ruudut[paikka.0][paikka.1] = Some(erikois);
                luodut.push(paikka);
            }
        }
        let valissa = self.ruudut;
        let pudotukset = self.pudota(arpa);
        Some(Kierros {
            ennen,
            pisteet: RUUDUN_PISTEET * poistuneet.len() as u32 * ketju * self.taso,
            poistuneet,
            valissa,
            luodut,
            rajahdykset,
            pudotukset,
        })
    }

    /// Pommit räjäyttävät naapurinsa ja osuman saanut kaikkipalikka vie satunnaisen värin.
    /// Palauttaa räjähtäneiden pommien paikat.
    fn laajenna_poistot(
        &self,
        poistuu: &mut Poistot,
        laukeamatta: &[Paikka],
        arpa: &mut Arpa,
    ) -> Vec<Paikka> {
        let mut rajahdykset = Vec::new();
        let mut jono: Vec<Paikka> = kaikki_paikat().filter(|p| poistuu[p.0][p.1]).collect();
        while let Some(p) = jono.pop() {
            match self.palikka(p) {
                Some(Palikka::Pommi(_)) => {
                    rajahdykset.push(p);
                    let rivit = p.0.saturating_sub(1)..(p.0 + 2).min(SIVU);
                    let sarakkeet = p.1.saturating_sub(1)..(p.1 + 2).min(SIVU);
                    let ymparisto = rivit.flat_map(|r| sarakkeet.clone().map(move |s| (r, s)));
                    for q in ymparisto {
                        if !poistuu[q.0][q.1] {
                            poistuu[q.0][q.1] = true;
                            jono.push(q);
                        }
                    }
                }
                Some(Palikka::Kaikki) if !laukeamatta.contains(&p) => {
                    let jaljella: Vec<u8> = kaikki_paikat()
                        .filter(|q| !poistuu[q.0][q.1])
                        .filter_map(|q| vari_kohdassa(&self.ruudut, q))
                        .collect();
                    if jaljella.is_empty() {
                        continue;
                    }
                    let vari = jaljella[arpa.alle(jaljella.len())];
                    for q in kaikki_paikat() {
                        if !poistuu[q.0][q.1] && vari_kohdassa(&self.ruudut, q) == Some(vari) {
                            poistuu[q.0][q.1] = true;
                            jono.push(q);
                        }
                    }
                }
                _ => {}
            }
        }
        rajahdykset
    }

    /// Täyttää tyhjät sarakkeittain ja kertoo, mikä putosi mistä.
    fn pudota(&mut self, arpa: &mut Arpa) -> Vec<Pudotus> {
        let maara = varien_maara(self.taso);
        let mut pudotukset = Vec::new();
        for sarake in 0..SIVU {
            let mut tyhjia = 0;
            for rivi in (0..SIVU).rev() {
                match self.ruudut[rivi][sarake] {
                    None => tyhjia += 1,
                    Some(palikka) if tyhjia > 0 => {
                        self.ruudut[rivi + tyhjia][sarake] = Some(palikka);
                        self.ruudut[rivi][sarake] = None;
                        pudotukset.push(Pudotus {
                            mista_rivi: rivi as i32,
                            mihin: (rivi + tyhjia, sarake),
                            palikka,
                        });
                    }
                    Some(_) => {}
                }
            }
            for rivi in 0..tyhjia {
                let palikka = Palikka::Tavallinen(arpa.alle(maara) as u8);
                self.ruudut[rivi][sarake] = Some(palikka);
                pudotukset.push(Pudotus {
                    mista_rivi: rivi as i32 - tyhjia as i32,
                    mihin: (rivi, sarake),
                    palikka,
                });
            }
        }
        pudotukset
    }
}
