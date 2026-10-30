use klassikot8000::Arpa;
use klassikot8000::tiilirikko::{
    ASKEL_MS, ELAMIA, KENTTIA, KORKEUS, LEVEYS, MAILA_L, MAILA_Y, PALLON_SADE, RIVEJA, SARAKKEITA,
    TIILET_Y, TIILI_K, TIILI_L, Tapahtuma, Tiilipeli, Tila,
};

/// Peli, jossa on vain yksi tiili kaukana pallosta: kenttä ei vaihdu kesken testin.
fn lahes_tyhja() -> Tiilipeli {
    let mut peli = Tiilipeli::new(0);
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.tila = Tila::Lennossa;
    peli
}

/// Kutsuu pelin `laukaise`-metodia, jotta sisäinen vauhti on asetettu.
fn lennossa_tyhjalla() -> Tiilipeli {
    let mut peli = lahes_tyhja();
    peli.tila = Tila::Kiinni;
    peli.laukaise(&mut Arpa::new(1));
    peli
}

#[test]
fn jokaisessa_kentassa_on_tiilia_ja_ylisuuri_numero_rajataan() {
    for kentta in 0..KENTTIA {
        let peli = Tiilipeli::new(kentta);
        assert!(peli.tiilia_jaljella() > 0, "{kentta}");
        assert_eq!(peli.tila, Tila::Kiinni);
        assert_eq!(peli.elamat, ELAMIA);
    }
    assert_eq!(Tiilipeli::new(99).kentta, KENTTIA - 1);
}

#[test]
fn maila_pysyy_kentalla_ja_kiinni_oleva_pallo_seuraa() {
    let mut peli = Tiilipeli::new(0);
    peli.aseta_maila(-100.0);
    assert_eq!(peli.maila_x, MAILA_L / 2.0);
    peli.aseta_maila(10_000.0);
    assert_eq!(peli.maila_x, LEVEYS - MAILA_L / 2.0);
    assert_eq!(peli.pallo.0, peli.maila_x);

    let alku = peli.maila_x;
    peli.askel(-1);
    assert!(peli.maila_x < alku);
    assert_eq!(peli.pallo.0, peli.maila_x);
}

#[test]
fn laukaisu_lahtee_ylos_vinoon_eika_toistu_lennossa() {
    for siemen in 0..100 {
        let mut arpa = Arpa::new(siemen);
        let mut peli = Tiilipeli::new(0);
        peli.laukaise(&mut arpa);
        assert_eq!(peli.tila, Tila::Lennossa);
        assert!(peli.nopeus.1 < 0.0);
        assert!(peli.nopeus.0 != 0.0);
        let nopeus = peli.nopeus;
        peli.laukaise(&mut arpa);
        assert_eq!(peli.nopeus, nopeus);
    }
}

#[test]
fn seinat_kaantavat_pallon() {
    let mut peli = lennossa_tyhjalla();
    peli.pallo = (PALLON_SADE + 0.1, 300.0);
    peli.nopeus = (-300.0, -100.0);
    assert_eq!(peli.askel(0), [Tapahtuma::Seina]);
    assert!(peli.nopeus.0 > 0.0);

    peli.pallo = (LEVEYS - PALLON_SADE - 0.1, 300.0);
    peli.nopeus = (300.0, -100.0);
    assert_eq!(peli.askel(0), [Tapahtuma::Seina]);
    assert!(peli.nopeus.0 < 0.0);

    peli.pallo = (300.0, PALLON_SADE + 0.1);
    peli.nopeus = (100.0, -300.0);
    assert_eq!(peli.askel(0), [Tapahtuma::Seina]);
    assert!(peli.nopeus.1 > 0.0);
}

#[test]
fn mailan_laita_ohjaa_palloa_ja_vauhti_sailyy() {
    for (poikkeama, suunta) in [(-40.0, -1.0), (40.0, 1.0)] {
        let mut peli = lennossa_tyhjalla();
        let vauhti = peli.nopeus.0.hypot(peli.nopeus.1);
        peli.aseta_maila(300.0);
        peli.pallo = (300.0 + poikkeama, MAILA_Y - PALLON_SADE - 0.1);
        peli.nopeus = (0.0, vauhti);
        assert_eq!(peli.askel(0), [Tapahtuma::Maila]);
        assert!(peli.nopeus.1 < 0.0);
        assert!(peli.nopeus.0 * suunta > 0.0);
        assert!((peli.nopeus.0.hypot(peli.nopeus.1) - vauhti).abs() < 0.01);
    }
}

#[test]
fn nouseva_pallo_kulkee_mailan_lapi() {
    let mut peli = lennossa_tyhjalla();
    peli.aseta_maila(300.0);
    peli.pallo = (300.0, MAILA_Y + 5.0);
    peli.nopeus = (0.0, -300.0);
    assert!(peli.askel(0).is_empty());
    assert!(peli.nopeus.1 < 0.0);
}

#[test]
fn kestava_tiili_vaatii_useamman_osuman_ja_kaantaa_pallon() {
    let mut peli = lennossa_tyhjalla();
    peli.tiilet[3][5] = 2;
    let y = TIILET_Y + 4.0 * TIILI_K + PALLON_SADE + 0.5;
    let x = 5.5 * TIILI_L;
    let lauo = |peli: &mut Tiilipeli| {
        peli.pallo = (x, y);
        peli.nopeus = (0.0, -300.0);
        peli.askel(0)
    };
    assert_eq!(
        lauo(&mut peli),
        [Tapahtuma::Tiili {
            rivi: 3,
            sarake: 5,
            rikki: false
        }]
    );
    assert!(peli.nopeus.1 > 0.0);
    assert_eq!(peli.pisteet, 10);
    assert_eq!(
        lauo(&mut peli),
        [Tapahtuma::Tiili {
            rivi: 3,
            sarake: 5,
            rikki: true
        }]
    );
    assert_eq!(peli.tiilet[3][5], 0);
    assert_eq!(peli.pisteet, 20);
}

#[test]
fn sivusta_tuleva_osuma_kaantaa_vaakasuunnan_ja_vauhti_kasvaa() {
    let mut peli = lennossa_tyhjalla();
    peli.tiilet[3][5] = 1;
    let vauhti = peli.nopeus.0.hypot(peli.nopeus.1);
    peli.pallo = (5.0 * TIILI_L - PALLON_SADE - 0.5, TIILET_Y + 3.5 * TIILI_K);
    let suunta = (vauhti * 0.8, vauhti * 0.6);
    peli.nopeus = suunta;
    peli.askel(0);
    assert!(peli.nopeus.0 < 0.0);
    assert!(peli.nopeus.1 > 0.0);
    assert!(peli.nopeus.0.hypot(peli.nopeus.1) > vauhti);
}

#[test]
fn pallon_menetys_kuluttaa_elaman_ja_viimeinen_paattaa_pelin() {
    let mut peli = lennossa_tyhjalla();
    let pudota = |peli: &mut Tiilipeli| {
        peli.pallo = (LEVEYS / 2.0, KORKEUS + PALLON_SADE + 0.1);
        peli.nopeus = (0.0, 300.0);
        peli.askel(0)
    };
    assert_eq!(pudota(&mut peli), [Tapahtuma::Menetys]);
    assert_eq!(peli.tila, Tila::Kiinni);
    assert_eq!(peli.elamat, ELAMIA - 1);

    peli.tila = Tila::Lennossa;
    assert_eq!(pudota(&mut peli), [Tapahtuma::Menetys]);
    peli.tila = Tila::Lennossa;
    assert_eq!(pudota(&mut peli), [Tapahtuma::Ohi]);
    assert_eq!(peli.tila, Tila::Ohi);
    assert_eq!(peli.elamat, 0);

    // päättynyt peli ei enää etene
    assert!(peli.askel(1).is_empty());
}

#[test]
fn viimeinen_tiili_avaa_seuraavan_kentan_ja_viimeisessa_kentassa_voittaa() {
    let mut peli = Tiilipeli::new(KENTTIA - 2);
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.laukaise(&mut Arpa::new(1));
    let viimeistele = |peli: &mut Tiilipeli| {
        peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
        peli.tiilet[0][0] = 1;
        peli.pallo = (TIILI_L / 2.0, TIILET_Y + TIILI_K + PALLON_SADE + 0.5);
        peli.nopeus = (0.0, -300.0);
        peli.askel(0)
    };
    let tapahtumat = viimeistele(&mut peli);
    assert!(matches!(
        tapahtumat.last(),
        Some(Tapahtuma::KenttaLapi { .. })
    ));
    assert_eq!(peli.kentta, KENTTIA - 1);
    assert_eq!(peli.tila, Tila::Kiinni);
    assert!(peli.tiilia_jaljella() > 0);

    peli.laukaise(&mut Arpa::new(1));
    let tapahtumat = viimeistele(&mut peli);
    assert!(matches!(tapahtumat.last(), Some(Tapahtuma::Voitto { .. })));
    assert_eq!(peli.tila, Tila::Voitto);
}

/// Rikkoo kentän ainoan tiilen, kun kentällä on kulunut `kulunut_ms`. Palauttaa bonuksen ja pisteiden kasvun.
fn lapaisy(kentta: usize, kulunut_ms: u32) -> (u32, u32) {
    let mut peli = Tiilipeli::new(kentta);
    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.laukaise(&mut Arpa::new(1));
    // Osuman askel kuluttaa vielä oman aikansa.
    peli.kentan_ms = kulunut_ms.saturating_sub(ASKEL_MS);
    peli.pallo = (TIILI_L / 2.0, TIILET_Y + TIILI_K + PALLON_SADE + 0.5);
    peli.nopeus = (0.0, -300.0);
    let tapahtumat = peli.askel(0);
    let bonus = match tapahtumat.last() {
        Some(Tapahtuma::KenttaLapi { bonus } | Tapahtuma::Voitto { bonus }) => *bonus,
        muu => panic!("{muu:?}"),
    };
    (bonus, peli.pisteet)
}

#[test]
fn nopeampi_lapaisy_antaa_enemman_pisteita() {
    let (alussa, pisteet) = lapaisy(0, 0);
    assert!((990..1000).contains(&alussa), "{alussa}");
    assert_eq!(pisteet, 10 + alussa);
    assert_eq!(lapaisy(0, 120_000).0, 500);
    assert_eq!(lapaisy(0, 240_000).0, 0);
    assert_eq!(lapaisy(0, 999_000).0, 0);
    assert!(lapaisy(0, 60_000).1 > lapaisy(0, 180_000).1);
    assert_eq!(lapaisy(KENTTIA - 1, 120_000).0, 500 * KENTTIA as u32);
}

#[test]
fn kentan_kello_kay_vain_lennossa_ja_nollautuu_kentan_vaihtuessa() {
    let mut peli = Tiilipeli::new(0);
    peli.askel(0);
    assert_eq!(peli.kentan_ms, 0);
    peli.laukaise(&mut Arpa::new(1));
    peli.askel(0);
    assert_eq!(peli.kentan_ms, ASKEL_MS);

    peli.tiilet = [[0; SARAKKEITA]; RIVEJA];
    peli.tiilet[0][0] = 1;
    peli.pallo = (TIILI_L / 2.0, TIILET_Y + TIILI_K + PALLON_SADE + 0.5);
    peli.nopeus = (0.0, -300.0);
    peli.askel(0);
    assert_eq!(peli.kentta, 1);
    assert_eq!(peli.kentan_ms, 0);
}

fn pallo_tiilen_sisalla(peli: &Tiilipeli) -> bool {
    let (x, y) = peli.pallo;
    (0..RIVEJA).any(|rivi| {
        (0..SARAKKEITA).any(|sarake| {
            let vasen = sarake as f32 * TIILI_L;
            let yla = TIILET_Y + rivi as f32 * TIILI_K;
            peli.tiilet[rivi][sarake] > 0
                && x + PALLON_SADE < vasen + TIILI_L + PALLON_SADE
                && x - PALLON_SADE > vasen - PALLON_SADE
                && y + PALLON_SADE < yla + TIILI_K + PALLON_SADE
                && y - PALLON_SADE > yla - PALLON_SADE
        })
    })
}

/// Automaatti pitää mailan pallon alla vaihtelevalla poikkeamalla, joten se pelaa koko pelin.
#[test]
fn automaatti_lapaisee_kaikki_kentat_eika_pallo_karkaa() {
    for siemen in 0..20 {
        let mut arpa = Arpa::new(siemen);
        let mut peli = Tiilipeli::new(0);
        let mut poikkeama = 0.0;
        let mut askelia = 0u32;
        while peli.tila != Tila::Voitto {
            askelia += 1;
            assert!(askelia < 20_000_000, "siemen {siemen}: peli ei pääty");
            if peli.tila == Tila::Kiinni {
                peli.laukaise(&mut arpa);
            }
            peli.aseta_maila(peli.pallo.0 + poikkeama);
            for tapahtuma in peli.askel(0) {
                if tapahtuma == Tapahtuma::Maila {
                    poikkeama = arpa.alle(81) as f32 - 40.0;
                }
                assert_ne!(tapahtuma, Tapahtuma::Menetys, "siemen {siemen}");
            }
            assert!(!pallo_tiilen_sisalla(&peli), "siemen {siemen}");
            assert!(
                (PALLON_SADE..=LEVEYS - PALLON_SADE).contains(&peli.pallo.0),
                "siemen {siemen}: x {}",
                peli.pallo.0
            );
            assert!(
                peli.pallo.1 >= PALLON_SADE && peli.pallo.1 <= MAILA_Y,
                "siemen {siemen}"
            );
        }
        assert_eq!(peli.tiilia_jaljella(), 0);
        assert_eq!(peli.kentta, KENTTIA - 1);
    }
}
