//! Upottaa ikonin Windowsin exeen. Linkkeri ottaa .res-tiedoston suoraan, ja sen muoto on niin
//! yksinkertainen, että se syntyy tässä ilman rc.exe:ä tai riippuvuuksia.

use std::env;
use std::fs;
use std::path::Path;

const IKONI: &str = "paketointi/ikoni.ico";
const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
const MOVEABLE_PURE_DISCARDABLE: u16 = 0x1030;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={IKONI}");
    let alusta = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let ymparisto = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if alusta != "windows" || ymparisto != "msvc" {
        return;
    }
    let ico = fs::read(IKONI).expect("ikoni.ico puuttuu");
    let kansio = env::var("OUT_DIR").expect("cargo asettaa OUT_DIR:n");
    let polku = Path::new(&kansio).join("ikoni.res");
    fs::write(&polku, resurssit(&ico)).expect("ikoni.res ei kirjoitu");
    println!("cargo:rustc-link-arg-bins={}", polku.display());
}

/// Muuntaa .ico-tiedoston resursseiksi: yksi RT_ICON joka koolle ja RT_GROUP_ICON hakemistoksi.
fn resurssit(ico: &[u8]) -> Vec<u8> {
    let luku16 = |i: usize| u16::from_le_bytes([ico[i], ico[i + 1]]);
    let luku32 = |i: usize| u32::from_le_bytes([ico[i], ico[i + 1], ico[i + 2], ico[i + 3]]);
    let maara = luku16(4);

    let mut hakemisto = vec![0, 0, 1, 0];
    hakemisto.extend_from_slice(&maara.to_le_bytes());
    // Tiedoston alussa on aina tyhjä tunnistemerkintä.
    let mut res = Vec::new();
    lisaa(&mut res, 0, 0, 0, &[]);
    for n in 0..maara {
        let kohta = 6 + 16 * usize::from(n);
        let (koko, paikka) = (luku32(kohta + 8), luku32(kohta + 12) as usize);
        let tunnus = n + 1;
        lisaa(
            &mut res,
            RT_ICON,
            tunnus,
            MOVEABLE_PURE_DISCARDABLE,
            &ico[paikka..paikka + koko as usize],
        );
        // Hakemiston rivi on .ico:n rivi, jossa tiedostopaikan tilalla on resurssin tunnus.
        hakemisto.extend_from_slice(&ico[kohta..kohta + 12]);
        hakemisto.extend_from_slice(&tunnus.to_le_bytes());
    }
    lisaa(
        &mut res,
        RT_GROUP_ICON,
        1,
        MOVEABLE_PURE_DISCARDABLE,
        &hakemisto,
    );
    res
}

fn lisaa(res: &mut Vec<u8>, tyyppi: u16, nimi: u16, liput: u16, data: &[u8]) {
    res.extend_from_slice(&(data.len() as u32).to_le_bytes());
    res.extend_from_slice(&32u32.to_le_bytes());
    for tunnus in [0xFFFF, tyyppi, 0xFFFF, nimi] {
        res.extend_from_slice(&u16::to_le_bytes(tunnus));
    }
    res.extend_from_slice(&0u32.to_le_bytes());
    res.extend_from_slice(&liput.to_le_bytes());
    // Kieli: englanti (Yhdysvallat) on resurssien tavallinen oletus.
    res.extend_from_slice(&0x0409u16.to_le_bytes());
    res.extend_from_slice(&[0; 8]);
    res.extend_from_slice(data);
    res.resize(res.len().next_multiple_of(4), 0);
}
