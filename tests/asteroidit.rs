use klassikot8000::Arpa;
use klassikot8000::asteroidit::{
    AALLON_TAUKO_MS, ALUKSEN_SADE, ASKEL_MS, Ammus, Avaruus, ELAMIA, KORKEUS, Kivi, Koko, LEVEYS,
    LISAELAMAN_KASVU, LISAELAMAN_VALI, ODOTUS_MS, OMIA_AMMUKSIA, Ohjaus, PIENEN_LAUTASEN_RAJA,
    SUOJA_MS, SUURIN_PISTEMAARA, Tapahtuma, Tila, etaisyys,
};

fn kivi(paikka: (f32, f32), koko: Koko) -> Kivi {
    Kivi {
        paikka,
        nopeus: (0.0, 0.0),
        koko,
        kulma: 0.0,
        pyorinta: 0.0,
        muoto: 0,
    }
}

/// Peli ilman muita kiviä kuin kaukana aluksesta oleva yksi: aalto ei vaihdu kesken testin.
fn rauhallinen(arpa: &mut Arpa) -> Avaruus {
    let mut peli = Avaruus::new(0, arpa);
    peli.kivet = vec![kivi((40.0, 40.0), Koko::Pieni)];
    peli.lautasen_viive_ms = u32::MAX;
    peli
}

fn laukaus(paikka: (f32, f32), oma: bool) -> Ammus {
    Ammus {
        paikka,
        nopeus: (0.0, 0.0),
        jaljella_ms: 500,
        oma,
    }
}

#[test]
fn aalto_alkaa_laidalta_kaukana_aluksesta_ja_kivia_tulee_lisaa() {
    for siemen in 0..200 {
        let mut arpa = Arpa::new(siemen);
        for aalto in 0..12 {
            let peli = Avaruus::new(aalto, &mut arpa);
            assert_eq!(peli.kivet.len(), (4 + aalto).min(11));
            for kivi in &peli.kivet {
                assert_eq!(kivi.koko, Koko::Iso);
                assert!(etaisyys(kivi.paikka, peli.alus.paikka) >= 150.0);
                assert!(kivi.paikka.0 == 0.0 || kivi.paikka.1 == 0.0 || kivi.paikka.0 >= 330.0);
            }
        }
    }
}

#[test]
fn kaikki_liikkuu_kentan_sisalla_reunojen_yli() {
    let mut arpa = Arpa::new(5);
    let mut peli = Avaruus::new(10, &mut arpa);
    peli.lautasen_viive_ms = 100;
    let ohjaus = Ohjaus {
        kaanto: 1,
        tyonto: true,
        tahtain: None,
    };
    for i in 0..20_000 {
        peli.askel(ohjaus, &mut arpa);
        if i % 50 == 0 {
            peli.ammu();
        }
        let sisalla =
            |(x, y): (f32, f32)| (0.0..=LEVEYS).contains(&x) && (0.0..=KORKEUS).contains(&y);
        assert!(sisalla(peli.alus.paikka));
        assert!(peli.kivet.iter().all(|k| sisalla(k.paikka)));
        assert!(peli.ammukset.iter().all(|a| sisalla(a.paikka)));
    }
}

#[test]
fn etaisyys_lasketaan_reunan_yli() {
    assert_eq!(etaisyys((5.0, 100.0), (LEVEYS - 5.0, 100.0)), 10.0);
    assert_eq!(etaisyys((100.0, 5.0), (100.0, KORKEUS - 5.0)), 10.0);
}

#[test]
fn ammuksia_on_rajallisesti_ja_ne_haviavat_aikanaan() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    for _ in 0..OMIA_AMMUKSIA {
        assert!(peli.ammu());
    }
    assert!(!peli.ammu());
    for _ in 0..1000 / ASKEL_MS {
        peli.askel(Ohjaus::default(), &mut arpa);
    }
    assert!(peli.ammukset.is_empty());
    assert!(peli.ammu());
}

#[test]
fn ammuttu_kivi_hajoaa_pienemmiksi_ja_pisteet_kasvavat() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    for (koko, pisteet, sirpaleita) in [
        (Koko::Iso, 20, 2),
        (Koko::Keski, 50, 2),
        (Koko::Pieni, 100, 0),
    ] {
        peli.pisteet = 0;
        peli.kivet = vec![
            kivi((100.0, 100.0), koko),
            kivi((550.0, 450.0), Koko::Pieni),
        ];
        peli.ammukset = vec![laukaus((100.0, 100.0), true)];
        let tapahtumat = peli.askel(Ohjaus::default(), &mut arpa);
        assert!(matches!(tapahtumat[0], Tapahtuma::Kivi { koko: k, .. } if k == koko));
        assert_eq!(peli.pisteet, pisteet);
        assert_eq!(peli.kivet.len(), 1 + sirpaleita);
        assert!(peli.ammukset.is_empty());
    }
}

#[test]
fn lautasen_ammus_lentaa_kiven_lapi_mutta_tuhoaa_aluksen() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    peli.kivet = vec![kivi((100.0, 100.0), Koko::Iso)];
    peli.ammukset = vec![laukaus((100.0, 100.0), false)];
    assert!(peli.askel(Ohjaus::default(), &mut arpa).is_empty());
    assert_eq!(peli.kivet.len(), 1);

    peli.ammukset = vec![laukaus(peli.alus.paikka, false)];
    let tapahtumat = peli.askel(Ohjaus::default(), &mut arpa);
    assert!(matches!(tapahtumat[0], Tapahtuma::Menetys { .. }));
    assert_eq!(peli.tila, Tila::Tuho);
    assert_eq!(peli.elamat, ELAMIA - 1);
}

#[test]
fn tuhon_jalkeen_alus_palaa_keskelle_suojassa_ja_viimeinen_paattaa_pelin() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    let keski = peli.alus.paikka;
    peli.alus.paikka = (100.0, 300.0);
    peli.kivet = vec![kivi((100.0, 300.0), Koko::Iso), kivi(keski, Koko::Iso)];
    let tapahtumat = peli.askel(Ohjaus::default(), &mut arpa);
    assert!(
        tapahtumat
            .iter()
            .any(|t| matches!(t, Tapahtuma::Menetys { .. }))
    );
    assert!(!peli.ammu());
    assert!(!peli.hyppaa(&mut arpa));

    // Odotus lyheni jo tuhon askeleella.
    let mut palasi = false;
    for _ in 0..ODOTUS_MS / ASKEL_MS - 1 {
        palasi |= peli
            .askel(Ohjaus::default(), &mut arpa)
            .contains(&Tapahtuma::Palasi);
    }
    assert!(palasi);
    assert_eq!(peli.tila, Tila::Lennossa);
    assert_eq!(peli.alus.paikka, keski);
    assert!(peli.suoja_ms > 0);

    // Keskellä oleva kivi ei satuta suojan aikana, mutta suojan loputtua kyllä.
    for _ in 0..SUOJA_MS / ASKEL_MS - 1 {
        assert!(peli.askel(Ohjaus::default(), &mut arpa).is_empty());
    }
    peli.elamat = 1;
    let tapahtumat = peli.askel(Ohjaus::default(), &mut arpa);
    assert!(
        tapahtumat
            .iter()
            .any(|t| matches!(t, Tapahtuma::Ohi { .. }))
    );
    assert_eq!(peli.tila, Tila::Ohi);
    assert_eq!(peli.elamat, 0);
    assert!(peli.lautanen.is_none());
    assert!(!peli.ammu());

    // päättynyt peli ei enää tuota tapahtumia
    assert!(peli.askel(Ohjaus::default(), &mut arpa).is_empty());
}

#[test]
fn kiven_reuna_ratkaisee_osuman() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    let (x, y) = peli.alus.paikka;
    let raja = Koko::Pieni.sade() + ALUKSEN_SADE;
    peli.kivet = vec![kivi((x + raja + 0.5, y), Koko::Pieni)];
    assert!(peli.askel(Ohjaus::default(), &mut arpa).is_empty());
    peli.kivet = vec![kivi((x + raja - 0.5, y), Koko::Pieni)];
    assert!(!peli.askel(Ohjaus::default(), &mut arpa).is_empty());
}

#[test]
fn tyhja_kentta_tuo_tauon_jalkeen_seuraavan_aallon() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    peli.kivet.clear();
    for _ in 0..AALLON_TAUKO_MS / ASKEL_MS - 1 {
        assert!(peli.askel(Ohjaus::default(), &mut arpa).is_empty());
    }
    assert_eq!(
        peli.askel(Ohjaus::default(), &mut arpa),
        [Tapahtuma::UusiAalto]
    );
    assert_eq!(peli.aalto, 1);
    assert_eq!(peli.kivet.len(), 5);
}

#[test]
fn raja_tuo_lisaelaman_kerran_ja_pisteet_eivat_ylita_kattoa() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    let osuma = |peli: &mut Avaruus, arpa: &mut Arpa| {
        peli.kivet = vec![
            kivi((100.0, 100.0), Koko::Pieni),
            kivi((550.0, 450.0), Koko::Pieni),
        ];
        peli.ammukset = vec![laukaus((100.0, 100.0), true)];
        peli.askel(Ohjaus::default(), arpa)
    };
    peli.pisteet = LISAELAMAN_VALI - 50;
    assert!(osuma(&mut peli, &mut arpa).contains(&Tapahtuma::Lisaelama));
    assert_eq!(peli.elamat, ELAMIA + 1);
    assert!(!osuma(&mut peli, &mut arpa).contains(&Tapahtuma::Lisaelama));

    // Toinen lisäalus maksaa ensimmäistä enemmän.
    let toinen = 2 * LISAELAMAN_VALI + LISAELAMAN_KASVU;
    peli.pisteet = toinen - 101;
    assert!(!osuma(&mut peli, &mut arpa).contains(&Tapahtuma::Lisaelama));
    peli.pisteet = toinen - 100;
    assert!(osuma(&mut peli, &mut arpa).contains(&Tapahtuma::Lisaelama));
    assert_eq!(peli.elamat, ELAMIA + 2);

    peli.pisteet = SUURIN_PISTEMAARA - 1;
    osuma(&mut peli, &mut arpa);
    assert_eq!(peli.pisteet, SUURIN_PISTEMAARA);
}

#[test]
fn hyppy_siirtaa_aluksen_ja_vaatii_tauon() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    peli.alus.nopeus = (100.0, 0.0);
    let alku = peli.alus.paikka;
    assert!(peli.hyppaa(&mut arpa));
    assert_ne!(peli.alus.paikka, alku);
    assert_eq!(peli.alus.nopeus, (0.0, 0.0));
    assert!(!peli.hyppaa(&mut arpa));
    for _ in 0..1000 / ASKEL_MS {
        peli.askel(Ohjaus::default(), &mut arpa);
    }
    assert!(peli.hyppaa(&mut arpa));
}

#[test]
fn tyonto_kiihdyttaa_vauhti_on_rajattu_ja_kitka_hidastaa() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    peli.kivet.clear();
    let tyonto = Ohjaus {
        tyonto: true,
        ..Ohjaus::default()
    };
    let vauhti = |peli: &Avaruus| peli.alus.nopeus.0.hypot(peli.alus.nopeus.1);
    for _ in 0..400 {
        peli.askel(tyonto, &mut arpa);
    }
    assert!(peli.tyontaa);
    assert!(peli.alus.nopeus.1 < 0.0);
    assert!(vauhti(&peli) <= 380.0);
    let huippu = vauhti(&peli);
    for _ in 0..200 {
        peli.askel(Ohjaus::default(), &mut arpa);
    }
    assert!(!peli.tyontaa);
    assert!(vauhti(&peli) < huippu);
}

#[test]
fn tahtain_kaantaa_lyhinta_reittia_ja_pysahtyy_kohteeseen() {
    let mut arpa = Arpa::new(1);
    let mut peli = rauhallinen(&mut arpa);
    peli.kivet.clear();
    let (x, y) = peli.alus.paikka;
    let ohjaus = Ohjaus {
        tahtain: Some((x + 100.0, y)),
        ..Ohjaus::default()
    };
    // alus osoittaa ylös, joten neljännesympyrä myötäpäivään riittää
    for _ in 0..80 {
        peli.askel(ohjaus, &mut arpa);
    }
    assert!(peli.alus.kulma.sin().abs() < 0.03, "{}", peli.alus.kulma);
    assert!(peli.alus.kulma.cos() > 0.99);
    let ohjaus = Ohjaus {
        kaanto: -1,
        ..ohjaus
    };
    let ennen = peli.alus.kulma;
    peli.askel(ohjaus, &mut arpa);
    assert_ne!(peli.alus.kulma, ennen);
}

#[test]
fn lautanen_ylittaa_kentan_ampuu_ja_poistuu() {
    let mut arpa = Arpa::new(9);
    let mut peli = rauhallinen(&mut arpa);
    peli.suoja_ms = u32::MAX;
    peli.lautasen_viive_ms = ASKEL_MS;
    let mut ampui = false;
    peli.askel(Ohjaus::default(), &mut arpa);
    assert!(peli.lautanen.is_some());
    assert!(!peli.lautanen.unwrap().pieni);
    for _ in 0..3000 {
        ampui |= peli
            .askel(Ohjaus::default(), &mut arpa)
            .contains(&Tapahtuma::LautanenAmpui);
        if peli.lautanen.is_none() {
            break;
        }
    }
    assert!(ampui);
    assert!(peli.lautanen.is_none());
    assert!(peli.lautasen_viive_ms > 0);
}

#[test]
fn lautaset_yltyvat_aaltojen_myota_rajaan_asti() {
    for siemen in 0..100 {
        let mut arpa = Arpa::new(siemen);
        let alku = Avaruus::new(0, &mut arpa).lautasen_viive_ms;
        assert!((15_000..25_000).contains(&alku));
        let myohemmin = Avaruus::new(5, &mut arpa).lautasen_viive_ms;
        assert!((10_000..20_000).contains(&myohemmin));
        assert_eq!(Avaruus::new(40, &mut arpa).lautasen_viive_ms, 6000);
    }
}

#[test]
fn pieni_lautanen_tulee_rajan_jalkeen_ja_antaa_enemman_pisteita() {
    let mut arpa = Arpa::new(9);
    let mut peli = rauhallinen(&mut arpa);
    peli.suoja_ms = u32::MAX;
    peli.pisteet = PIENEN_LAUTASEN_RAJA;
    peli.lautasen_viive_ms = ASKEL_MS;
    peli.askel(Ohjaus::default(), &mut arpa);
    let lautanen = peli.lautanen.unwrap();
    assert!(lautanen.pieni);

    peli.ammukset = vec![laukaus(lautanen.paikka, true)];
    let tapahtumat = peli.askel(Ohjaus::default(), &mut arpa);
    assert!(
        tapahtumat
            .iter()
            .any(|t| matches!(t, Tapahtuma::LautanenTuhoutui { .. }))
    );
    assert_eq!(peli.pisteet, PIENEN_LAUTASEN_RAJA + 1000);
    assert!(peli.lautanen.is_none());
}

/// Sama siemen ja ohjaus toistavat saman pelin.
#[test]
fn peli_on_toistettava() {
    let pelaa = || {
        let mut arpa = Arpa::new(42);
        let mut peli = Avaruus::new(2, &mut arpa);
        let ohjaus = Ohjaus {
            kaanto: 1,
            tyonto: false,
            tahtain: None,
        };
        for i in 0..30_000 {
            peli.askel(ohjaus, &mut arpa);
            if i % 40 == 0 {
                peli.ammu();
            }
        }
        (peli.pisteet, peli.aalto, peli.elamat, peli.kivet.len())
    };
    assert_eq!(pelaa(), pelaa());
    assert!(pelaa().0 > 0);
}
