use klassikot8000::Arpa;
use klassikot8000::palikka::{
    Paikka, Palikka, Palikkalauta, Ruudut, SIVU, SUURIN_TULOS, VAREJA, tason_tavoite, varien_maara,
    vierekkain,
};

const A: Option<Palikka> = Some(Palikka::Tavallinen(0));

/// Lauta ilman suoria ja siirtoja: neljän sarakkeen värijakso ei koskaan toista naapuriaan.
fn pohja() -> Ruudut {
    let mut ruudut = [[None; SIVU]; SIVU];
    for (r, rivi) in ruudut.iter_mut().enumerate() {
        for (s, ruutu) in rivi.iter_mut().enumerate() {
            let vari = (2 + (r % 3) * 2 + s % 2 + (s / 2) % 2 * 3) % 6;
            *ruutu = Some(Palikka::Tavallinen(vari as u8));
        }
    }
    ruudut
}

fn ei_suoria(ruudut: &Ruudut) -> bool {
    let vari = |r: usize, s: usize| ruudut[r][s].and_then(Palikka::vari);
    for r in 0..SIVU {
        for s in 0..SIVU {
            let Some(oma) = vari(r, s) else { continue };
            if s + 2 < SIVU && vari(r, s + 1) == Some(oma) && vari(r, s + 2) == Some(oma) {
                return false;
            }
            if r + 2 < SIVU && vari(r + 1, s) == Some(oma) && vari(r + 2, s) == Some(oma) {
                return false;
            }
        }
    }
    true
}

fn taysi(ruudut: &Ruudut) -> bool {
    ruudut.iter().flatten().all(Option::is_some)
}

#[test]
fn alkulaudalla_ei_ole_suoria_ja_siirto_loytyy() {
    for siemen in 0..200 {
        let mut arpa = Arpa::new(siemen);
        let lauta = Palikkalauta::new(&mut arpa);
        assert!(taysi(lauta.ruudut()));
        assert!(ei_suoria(lauta.ruudut()), "siemen {siemen}");
        let (a, b) = lauta.vihje().unwrap();
        assert!(vierekkain(a, b));
        for palikka in lauta.ruudut().iter().flatten().flatten() {
            assert!((palikka.vari().unwrap() as usize) < varien_maara(1));
        }
    }
}

#[test]
fn hyodyton_tai_kaukainen_vaihto_ei_muuta_lautaa() {
    let mut arpa = Arpa::new(1);
    let mut lauta = Palikkalauta::ruuduista(pohja());
    assert!(ei_suoria(lauta.ruudut()));
    assert_eq!(lauta.vaihda((0, 0), (0, 1), &mut arpa), None);
    assert_eq!(lauta.vaihda((0, 0), (0, 2), &mut arpa), None);
    assert_eq!(lauta.vaihda((0, 0), (1, 1), &mut arpa), None);
    assert_eq!(lauta.vaihda((7, 7), (7, 8), &mut arpa), None);
    assert_eq!(lauta.ruudut(), &pohja());
    assert_eq!(lauta.pisteet, 0);
}

#[test]
fn kolmen_suora_poistuu_ja_palikat_putoavat() {
    let mut ruudut = pohja();
    let ylla = ruudut[1][3];
    ruudut[2][0] = A;
    ruudut[2][1] = A;
    ruudut[3][2] = A;
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    let vaihto = lauta.vaihda((2, 2), (3, 2), &mut arpa).unwrap();

    let eka = &vaihto.kierrokset[0];
    assert_eq!(eka.poistuneet, [(2, 0), (2, 1), (2, 2)]);
    assert_eq!(eka.pisteet, 30);
    assert!(eka.luodut.is_empty());
    assert_eq!(eka.pudotukset.len(), 9);
    // yläpuolelta tulee kolme uutta palikkaa, ja vanhat siirtyvät rivin alemmas
    let uudet = eka.pudotukset.iter().filter(|p| p.mista_rivi < 0).count();
    assert_eq!(uudet, 3);
    assert!(eka.pudotukset.iter().all(|p| p.mihin.0 <= 2));
    // sarake 3 ei liikkunut
    assert_eq!(eka.valissa[1][3], ylla);
    assert!(eka.pudotukset.iter().all(|p| p.mihin.1 < 3));
    assert!(lauta.pisteet >= 30);
    assert_eq!(vaihto.pisteet, lauta.pisteet);
    assert!(taysi(lauta.ruudut()));
    assert!(ei_suoria(lauta.ruudut()));
}

#[test]
fn kierrokset_kertovat_kaiken_toistoa_varten() {
    let mut arpa = Arpa::new(11);
    let mut lauta = Palikkalauta::new(&mut arpa);
    for _ in 0..300 {
        let Some((a, b)) = lauta.vihje() else { break };
        let vaihto = lauta.vaihda(a, b, &mut arpa).unwrap();
        let mut ennen_lauta = None;
        for kierros in &vaihto.kierrokset {
            if let Some(ennen) = ennen_lauta {
                assert_eq!(kierros.ennen, ennen);
            }
            // toistettu putoaminen päätyy samaan laudaksi kuin logiikka
            let mut kuva = kierros.valissa;
            for p in &kierros.pudotukset {
                if p.mista_rivi >= 0 {
                    kuva[p.mista_rivi as usize][p.mihin.1] = None;
                }
            }
            for p in &kierros.pudotukset {
                kuva[p.mihin.0][p.mihin.1] = Some(p.palikka);
            }
            assert!(taysi(&kuva));
            ennen_lauta = Some(kuva);
        }
        assert_eq!(ennen_lauta.as_ref(), Some(lauta.ruudut()));
    }
}

#[test]
fn neljan_suora_jattaa_pommin_vaihdon_kohdalle() {
    let mut ruudut = pohja();
    ruudut[2][0] = A;
    ruudut[2][1] = A;
    ruudut[2][3] = A;
    // pohjassa tässä on sama väri, ja suorasta tulisi viiden pituinen
    ruudut[2][4] = Some(Palikka::Tavallinen(1));
    ruudut[3][2] = A;
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    let vaihto = lauta.vaihda((3, 2), (2, 2), &mut arpa).unwrap();
    let eka = &vaihto.kierrokset[0];
    assert_eq!(eka.poistuneet.len(), 4);
    assert_eq!(eka.luodut, [(2, 2)]);
    assert_eq!(eka.valissa[2][2], Some(Palikka::Pommi(0)));
}

#[test]
fn l_muoto_on_yksi_ryhma_ja_antaa_pommin() {
    let mut ruudut = pohja();
    ruudut[2][0] = A;
    ruudut[2][1] = A;
    ruudut[3][2] = A;
    ruudut[4][2] = A;
    ruudut[2][3] = A;
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    let vaihto = lauta.vaihda((2, 3), (2, 2), &mut arpa).unwrap();
    let eka = &vaihto.kierrokset[0];
    assert_eq!(eka.poistuneet.len(), 5);
    assert_eq!(eka.valissa[2][2], Some(Palikka::Pommi(0)));
}

#[test]
fn viiden_suora_jattaa_kaikkipalikan() {
    let mut ruudut = pohja();
    for s in [0, 1, 3, 4] {
        ruudut[2][s] = A;
    }
    ruudut[3][2] = A;
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    let vaihto = lauta.vaihda((3, 2), (2, 2), &mut arpa).unwrap();
    assert_eq!(vaihto.kierrokset[0].valissa[2][2], Some(Palikka::Kaikki));
}

#[test]
fn poistuva_pommi_rajayttaa_ympariston() {
    let mut ruudut = pohja();
    ruudut[4][4] = Some(Palikka::Pommi(0));
    ruudut[4][5] = A;
    ruudut[4][7] = A;
    ruudut[5][6] = A;
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    let vaihto = lauta.vaihda((5, 6), (4, 6), &mut arpa).unwrap();
    let eka = &vaihto.kierrokset[0];
    assert_eq!(eka.rajahdykset, [(4, 4)]);
    // suoran neljä ruutua sekä pommin ympärys
    for r in 3..=5 {
        for s in 3..=5 {
            assert!(eka.poistuneet.contains(&(r, s)), "({r},{s})");
        }
    }
    assert!(eka.poistuneet.contains(&(4, 7)));
}

#[test]
fn pommi_laukaisee_toisen_pommin() {
    let mut ruudut = pohja();
    ruudut[4][1] = A;
    ruudut[4][2] = A;
    ruudut[5][3] = A;
    ruudut[3][1] = Some(Palikka::Pommi(4));
    ruudut[4][1] = Some(Palikka::Pommi(0));
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    let vaihto = lauta.vaihda((5, 3), (4, 3), &mut arpa).unwrap();
    let eka = &vaihto.kierrokset[0];
    assert!(eka.rajahdykset.contains(&(4, 1)));
    assert!(eka.rajahdykset.contains(&(3, 1)));
    assert!(eka.poistuneet.contains(&(2, 0)));
}

#[test]
fn kaikkipalikka_poistaa_naapurin_varin_ja_vaihto_kelpaa_aina() {
    let mut ruudut = pohja();
    ruudut[4][4] = Some(Palikka::Kaikki);
    let vari = ruudut[4][5].unwrap().vari().unwrap();
    let samat: Vec<Paikka> = (0..SIVU)
        .flat_map(|r| (0..SIVU).map(move |s| (r, s)))
        .filter(|&(r, s)| ruudut[r][s].and_then(Palikka::vari) == Some(vari))
        .collect();
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    assert_eq!(lauta.vihje(), Some(((3, 4), (4, 4))));
    let vaihto = lauta.vaihda((4, 4), (4, 5), &mut arpa).unwrap();
    let eka = &vaihto.kierrokset[0];
    assert_eq!(eka.poistuneet.len(), samat.len() + 1);
    assert!(eka.poistuneet.contains(&(4, 5)));
    assert!(eka.poistuneet.contains(&(4, 4)));
    assert!(taysi(lauta.ruudut()));
}

#[test]
fn kaksi_kaikkipalikkaa_tyhjentaa_laudan() {
    let mut ruudut = pohja();
    ruudut[4][4] = Some(Palikka::Kaikki);
    ruudut[4][5] = Some(Palikka::Kaikki);
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    let vaihto = lauta.vaihda((4, 4), (4, 5), &mut arpa).unwrap();
    assert_eq!(vaihto.kierrokset[0].poistuneet.len(), SIVU * SIVU);
    assert_eq!(vaihto.kierrokset[0].pudotukset.len(), SIVU * SIVU);
    assert!(taysi(lauta.ruudut()));
}

#[test]
fn ketju_kertoo_pisteet_ja_taso_nostaa_niita() {
    let mut arpa = Arpa::new(5);
    let mut lauta = Palikkalauta::new(&mut arpa);
    let mut ketjuja = 0;
    for _ in 0..500 {
        let Some((a, b)) = lauta.vihje() else { break };
        let taso = lauta.taso;
        let vaihto = lauta.vaihda(a, b, &mut arpa).unwrap();
        for (i, kierros) in vaihto.kierrokset.iter().enumerate() {
            let odotettu = 10 * kierros.poistuneet.len() as u32 * (i as u32 + 1) * taso;
            assert_eq!(kierros.pisteet, odotettu);
        }
        ketjuja += usize::from(vaihto.kierrokset.len() > 1);
        assert!(taysi(lauta.ruudut()));
        assert!(ei_suoria(lauta.ruudut()));
        assert_eq!(lauta.ohi, !lauta.on_siirtoja());
    }
    assert!(ketjuja > 0);
    assert!(lauta.taso > 1);
}

#[test]
fn taso_nousee_tavoitteesta_ja_varit_lisaantyvat() {
    assert_eq!(varien_maara(1), 6);
    assert_eq!(varien_maara(3), 7);
    assert_eq!(varien_maara(99), VAREJA);

    let mut ruudut = pohja();
    ruudut[2][0] = A;
    ruudut[2][1] = A;
    ruudut[3][2] = A;
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    lauta.tason_edistys = tason_tavoite(1) - 30;
    let vaihto = lauta.vaihda((2, 2), (3, 2), &mut arpa).unwrap();
    assert!(vaihto.uusi_taso);
    assert_eq!(lauta.taso, 2);
    assert_eq!(lauta.tason_edistys, vaihto.pisteet - 30);
}

#[test]
fn pisteet_eivat_ylita_rajaa() {
    let mut ruudut = pohja();
    ruudut[2][0] = A;
    ruudut[2][1] = A;
    ruudut[3][2] = A;
    let mut arpa = Arpa::new(3);
    let mut lauta = Palikkalauta::ruuduista(ruudut);
    lauta.pisteet = SUURIN_TULOS - 1;
    lauta.vaihda((2, 2), (3, 2), &mut arpa).unwrap();
    assert_eq!(lauta.pisteet, SUURIN_TULOS);
}

/// Nämä ovat pelin tasapainon rajat: peli ei saa loppua heti eikä jatkua ikuisesti.
#[test]
fn ahkera_pelaaja_lopulta_jumiutuu() {
    let pelit = 20;
    let mut siirrot_yhteensa = 0;
    let mut jumissa = 0;
    for siemen in 0..pelit {
        let mut arpa = Arpa::new(siemen);
        let mut lauta = Palikkalauta::new(&mut arpa);
        for _ in 0..20_000 {
            let Some((a, b)) = lauta.vihje() else { break };
            lauta.vaihda(a, b, &mut arpa).unwrap();
            siirrot_yhteensa += 1;
            if lauta.ohi {
                jumissa += 1;
                break;
            }
        }
    }
    println!("{siirrot_yhteensa} siirtoa, {jumissa}/{pelit} jumissa");
    assert!(siirrot_yhteensa / pelit >= 50, "{siirrot_yhteensa}");
    assert!(jumissa >= pelit / 2, "{jumissa}");
}
