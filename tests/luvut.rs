use klassikot8000::Arpa;
use klassikot8000::luvut::{Lukulauta, SIVU, Suunta};

const SUUNNAT: [Suunta; 4] = [Suunta::Ylos, Suunta::Alas, Suunta::Vasen, Suunta::Oikea];

fn rivi(luvut: [u32; 4]) -> Lukulauta {
    Lukulauta::ruuduista([luvut, [0; 4], [0; 4], [0; 4]])
}

fn ylarivi(lauta: &Lukulauta) -> [u32; 4] {
    [0, 1, 2, 3].map(|s| lauta.arvo((0, s)))
}

fn summa(lauta: &Lukulauta) -> u32 {
    (0..SIVU)
        .flat_map(|r| (0..SIVU).map(move |s| (r, s)))
        .map(|p| lauta.arvo(p))
        .sum()
}

#[test]
fn uusi_peli_alkaa_kahdella_luvulla() {
    for siemen in 0..200 {
        let lauta = Lukulauta::new(&mut Arpa::new(siemen));
        assert!(matches!(summa(&lauta), 4 | 6 | 8), "siemen {siemen}");
        assert_eq!(lauta.pisteet, 0);
    }
}

#[test]
fn yhdistyminen_noudattaa_saantoja() {
    // (ennen, vasemmalle siirrettynä, pisteet)
    for (ennen, jalkeen, pisteet) in [
        ([2, 2, 0, 0], [4, 0, 0, 0], 4),
        ([2, 0, 0, 2], [4, 0, 0, 0], 4),
        ([2, 2, 2, 0], [4, 2, 0, 0], 4),
        ([2, 2, 2, 2], [4, 4, 0, 0], 8),
        ([4, 2, 2, 0], [4, 4, 0, 0], 4),
        ([2, 4, 2, 0], [2, 4, 2, 0], 0),
    ] {
        let mut lauta = rivi(ennen);
        let mut arpa = Arpa::new(1);
        let siirto = lauta.siirra(Suunta::Vasen, &mut arpa);
        if ennen == [2, 4, 2, 0] {
            assert!(siirto.is_none());
            assert_eq!(ylarivi(&lauta), ennen);
            continue;
        }
        let siirto = siirto.unwrap();
        // uusi luku voi osua yläriville, joten se jätetään vertailusta pois
        let mut tulos = ylarivi(&lauta);
        if siirto.uusi.0 == 0 {
            tulos[siirto.uusi.1] = 0;
        }
        assert_eq!(tulos, jalkeen);
        assert_eq!(siirto.pisteet, pisteet);
        assert_eq!(lauta.pisteet, pisteet);
    }
}

#[test]
fn kaikki_suunnat_vievat_oikealle_reunalle() {
    for (suunta, kohde) in [
        (Suunta::Ylos, (0, 1)),
        (Suunta::Alas, (3, 1)),
        (Suunta::Vasen, (1, 0)),
        (Suunta::Oikea, (1, 3)),
    ] {
        let mut ruudut = [[0; 4]; 4];
        ruudut[1][1] = 8;
        let mut lauta = Lukulauta::ruuduista(ruudut);
        assert!(lauta.siirra(suunta, &mut Arpa::new(1)).is_some());
        assert_eq!(lauta.arvo(kohde), 8, "{suunta:?}");
    }
}

#[test]
fn liikkeet_kuvaavat_siirron() {
    let mut lauta = rivi([2, 2, 4, 0]);
    let siirto = lauta.siirra(Suunta::Vasen, &mut Arpa::new(1)).unwrap();
    assert_eq!(siirto.liikkeet.len(), 3);
    assert_eq!(siirto.yhdistyneet, [(0, 0)]);
    assert_eq!(siirto.liikkeet[2].mihin, (0, 1));
    assert_eq!(siirto.liikkeet[2].arvo, 4);
}

#[test]
fn taysi_lauta_ilman_pareja_paattaa_pelin() {
    let lukittu = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
    let mut lauta = Lukulauta::ruuduista(lukittu);
    assert!(!lauta.on_siirtoja());
    for suunta in SUUNNAT {
        assert!(lauta.siirra(suunta, &mut Arpa::new(1)).is_none());
    }

    let mut pari = lukittu;
    pari[3][3] = 4;
    assert!(Lukulauta::ruuduista(pari).on_siirtoja());
}

#[test]
fn satunnaispeli_sailyttaa_summan_ja_paattyy() {
    for siemen in 0..100 {
        let mut arpa = Arpa::new(siemen);
        let mut lauta = Lukulauta::new(&mut arpa);
        let mut pisteet = 0;
        while lauta.on_siirtoja() {
            let suunta = SUUNNAT[arpa.alle(4)];
            let ennen = summa(&lauta);
            if let Some(siirto) = lauta.siirra(suunta, &mut arpa) {
                let lisatty = lauta.arvo(siirto.uusi);
                assert!(lisatty == 2 || lisatty == 4);
                assert_eq!(summa(&lauta), ennen + lisatty);
                pisteet += siirto.pisteet;
                assert_eq!(lauta.pisteet, pisteet);
            } else {
                assert_eq!(summa(&lauta), ennen);
            }
        }
        assert!(lauta.suurin() >= 4);
    }
}
