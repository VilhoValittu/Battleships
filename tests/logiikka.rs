use laivanupotus::{Arpa, KOKO, LAIVAT, Laivasto, Laukaustulos, Ruudut, Ruutu, vastustaja};

fn kaikki_ruudut() -> impl Iterator<Item = Ruutu> {
    (0..KOKO * KOKO).filter_map(|i| Ruutu::new(i / KOKO, i % KOKO))
}

#[test]
fn laivasto_on_saantojen_mukainen_kaikilla_siemenilla() {
    for siemen in 0..500 {
        let laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let mut kaikki = Ruudut::default();
        for (laiva, (nimi, pituus)) in laivasto.laivat().iter().zip(LAIVAT) {
            assert_eq!(laiva.nimi, nimi);
            assert_eq!(laiva.ruudut.maara(), pituus);
            assert!(!laiva.ruudut.leikkaako(kaikki));
            kaikki = kaikki.yhdiste(laiva.ruudut);

            // suora ja yhtenäinen: toinen akseli on vakio, toisen vaihteluväli on pituus
            let ruudut: Vec<_> = kaikki_ruudut()
                .filter(|&r| laiva.ruudut.sisaltaa(r))
                .collect();
            let vali = |a: Vec<u8>| (a.iter().max().unwrap() - a.iter().min().unwrap()) as usize;
            let dr = vali(ruudut.iter().map(|r| r.rivi).collect());
            let ds = vali(ruudut.iter().map(|r| r.sarake).collect());
            assert!((dr, ds) == (0, pituus - 1) || (dr, ds) == (pituus - 1, 0));
        }
    }
}

#[test]
fn laivat_eivat_koske_toisiaan_edes_kulmittain() {
    for siemen in 0..5000 {
        let laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let ruudut = |i: usize| {
            let laiva = laivasto.laivat()[i];
            kaikki_ruudut().filter(move |&r| laiva.ruudut.sisaltaa(r))
        };
        for a in 0..LAIVAT.len() {
            for b in a + 1..LAIVAT.len() {
                for (ra, rb) in ruudut(a).flat_map(|ra| ruudut(b).map(move |rb| (ra, rb))) {
                    let etaisyys = ra.rivi.abs_diff(rb.rivi).max(ra.sarake.abs_diff(rb.sarake));
                    assert!(etaisyys > 1, "siemen {siemen}: {ra} ja {rb}");
                }
            }
        }
    }
}

#[test]
fn molemmat_suunnat_esiintyvat() {
    let mut suunnat = [false; 2];
    for siemen in 0..50 {
        let laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let laiva = laivasto.laivat()[0];
        let alku: Vec<_> = kaikki_ruudut()
            .filter(|&r| laiva.ruudut.sisaltaa(r))
            .take(2)
            .collect();
        suunnat[(alku[0].rivi == alku[1].rivi) as usize] = true;
    }
    assert_eq!(suunnat, [true, true]);
}

#[test]
fn laiva_uppoaa_vasta_viimeisesta_ruudusta_ja_voitto_17_osumasta() {
    for siemen in 0..100 {
        let mut laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let mut osumia = 0;
        let mut upotettuja = 0;
        for ruutu in kaikki_ruudut() {
            let oli_laiva = laivasto.on_laiva(ruutu);
            match laivasto.kasittele_laukaus(ruutu).unwrap() {
                Laukaustulos::Ohi => assert!(!oli_laiva),
                Laukaustulos::Osuma => osumia += 1,
                Laukaustulos::Upposi(nimi) => {
                    osumia += 1;
                    upotettuja += 1;
                    let laiva = laivasto.laivat().iter().find(|l| l.nimi == nimi).unwrap();
                    assert!(laiva.ruudut.sisaltaa(ruutu));
                    assert!(laiva.ruudut.on_osajoukko(laivasto.ammutut()));
                }
            }
            assert_eq!(laivasto.kaikki_upotettu(), osumia == 17);
        }
        assert_eq!((osumia, upotettuja), (17, 5));
    }
}

#[test]
fn upotettujen_laskuri_seuraa_uponneita_laivoja() {
    for siemen in 0..100 {
        let mut laivasto = Laivasto::satunnainen(&mut Arpa::new(siemen));
        let mut uponneet = Vec::new();
        for ruutu in kaikki_ruudut() {
            if let Some(Laukaustulos::Upposi(nimi)) = laivasto.kasittele_laukaus(ruutu) {
                uponneet.push(nimi);
            }
            assert_eq!(laivasto.upotettuja(), uponneet.len());
            for laiva in laivasto.laivat() {
                assert_eq!(laivasto.on_upotettu(laiva), uponneet.contains(&laiva.nimi));
            }
        }
    }
}

#[test]
fn sama_ruutu_hylataan_toisella_kerralla() {
    let mut laivasto = Laivasto::satunnainen(&mut Arpa::new(1));
    let ruutu = Ruutu::new(4, 4).unwrap();
    assert!(laivasto.kasittele_laukaus(ruutu).is_some());
    assert_eq!(laivasto.kasittele_laukaus(ruutu), None);
    assert_eq!(laivasto.ammutut().maara(), 1);
}

#[test]
fn vastustaja_ei_ammu_samaan_ja_kattaa_koko_laudan() {
    let mut arpa = Arpa::new(3);
    let mut ammutut = Ruudut::default();
    for _ in 0..KOKO * KOKO {
        let ruutu = vastustaja::valitse(ammutut, &mut arpa).unwrap();
        assert!(!ammutut.sisaltaa(ruutu));
        ammutut.lisaa(ruutu);
    }
    assert_eq!(vastustaja::valitse(ammutut, &mut arpa), None);
}
