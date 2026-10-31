#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod kaynnistin;
#[cfg(debug_assertions)]
mod kehitys;
mod pelit;
mod yhteinen;

use std::mem::{ManuallyDrop, take};
use std::path::Path;
#[cfg(debug_assertions)]
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use tinyaudio::{OutputDevice, OutputDeviceParameters, run_output_device};

#[cfg(debug_assertions)]
use kaynnistin::Kaynnissa;
use kaynnistin::netti::Netti;
use kaynnistin::{Kaynnistin, piirto};
#[cfg(debug_assertions)]
use klassikot8000::Arpa;
#[cfg(debug_assertions)]
use klassikot8000::asteroidit::Avaruus;
#[cfg(debug_assertions)]
use klassikot8000::luvut::Lukulauta;
#[cfg(debug_assertions)]
use klassikot8000::pasianssi::{self, KUNINGAS, Kortti, Maa};
#[cfg(debug_assertions)]
use klassikot8000::silmukka::{Laji, Otus, Suunta};
#[cfg(debug_assertions)]
use klassikot8000::tiilirikko::Tiilipeli;
#[cfg(debug_assertions)]
use pelit::laivanupotus::peli::Vaihe;
#[cfg(debug_assertions)]
use pelit::luvut::peli as luvut;
use pelit::luvut::tallennus;
#[cfg(debug_assertions)]
use pelit::pasianssi::peli as pasianssi_peli;
use yhteinen::aanet::{Mikseri, NAYTETAAJUUS};
use yhteinen::kieli;
use yhteinen::piirto::{Ikkunakuva, KORKEUS, LEVEYS, RUUDUN_KORKEUS, RUUDUN_LEVEYS, Sovitus};
use yhteinen::syote::Syote;

const PAIVITYKSIA_SEKUNNISSA: usize = 30;
const TOIMINTAPELIN_PAIVITYKSIA: usize = 60;
// Noin 36 ms: kytkin hiljentää äänet heti, mutta puskuri ei tyhjene kesken.
const AANIPUSKURI: usize = 400;
// Scale::FitScreen kertoo tämän suurimmalla kertoimella, joka mahtuu näytölle, ja vähentää
// yhden askeleen: ikkuna on 16:9 ja näytön reunaan jää tilaa otsikkopalkille ja tehtäväpalkille.
const IKKUNAN_POHJA: (usize, usize) = (160, 90);

fn aikasiemen() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |aika| aika.as_nanos() as u64)
}

fn nappain_syotteeksi(nappain: Key) -> Option<Syote> {
    Some(match nappain {
        Key::Up => Syote::Nuoli(-1, 0),
        Key::Down => Syote::Nuoli(1, 0),
        Key::Left => Syote::Nuoli(0, -1),
        Key::Right => Syote::Nuoli(0, 1),
        Key::Space | Key::Enter | Key::NumPadEnter => Syote::Ammu,
        Key::Escape => Syote::Peru,
        Key::M => Syote::VaihdaAanet,
        Key::A => Syote::Kirjain('A'),
        Key::B => Syote::Kirjain('B'),
        Key::C => Syote::Kirjain('C'),
        Key::D => Syote::Kirjain('D'),
        Key::E => Syote::Kirjain('E'),
        Key::F => Syote::Kirjain('F'),
        Key::G => Syote::Kirjain('G'),
        Key::H => Syote::Kirjain('H'),
        Key::I => Syote::Kirjain('I'),
        Key::J => Syote::Kirjain('J'),
        Key::K => Syote::Kirjain('K'),
        Key::L => Syote::Kirjain('L'),
        Key::N => Syote::Kirjain('N'),
        Key::O => Syote::Kirjain('O'),
        Key::P => Syote::Kirjain('P'),
        Key::Q => Syote::Kirjain('Q'),
        Key::R => Syote::Kirjain('R'),
        Key::S => Syote::Kirjain('S'),
        Key::T => Syote::Kirjain('T'),
        Key::U => Syote::Kirjain('U'),
        Key::V => Syote::Kirjain('V'),
        Key::W => Syote::Kirjain('W'),
        Key::X => Syote::Kirjain('X'),
        Key::Y => Syote::Kirjain('Y'),
        Key::Z => Syote::Kirjain('Z'),
        // Suomalaisen näppäimistön paikat: minifb lukee näppäimen sijainnin.
        Key::Semicolon => Syote::Kirjain('Ö'),
        Key::Apostrophe => Syote::Kirjain('Ä'),
        Key::Backspace => Syote::Pyyhi,
        Key::Key0 | Key::NumPad0 => Syote::Numero(0),
        Key::Key1 | Key::NumPad1 => Syote::Numero(1),
        Key::Key2 | Key::NumPad2 => Syote::Numero(2),
        Key::Key3 | Key::NumPad3 => Syote::Numero(3),
        Key::Key4 | Key::NumPad4 => Syote::Numero(4),
        Key::Key5 | Key::NumPad5 => Syote::Numero(5),
        Key::Key6 | Key::NumPad6 => Syote::Numero(6),
        Key::Key7 | Key::NumPad7 => Syote::Numero(7),
        Key::Key8 | Key::NumPad8 => Syote::Numero(8),
        Key::Key9 | Key::NumPad9 => Syote::Numero(9),
        _ => return None,
    })
}

fn paivitystaajuus(kaynnistin: &Kaynnistin) -> usize {
    if kaynnistin.toimintapeli() {
        TOIMINTAPELIN_PAIVITYKSIA
    } else {
        PAIVITYKSIA_SEKUNNISSA
    }
}

/// Avaa äänilaitteen. Jos sitä ei ole, peli toimii hiljaa.
fn avaa_aanilaite(mikseri: Arc<Mutex<Mikseri>>) -> Option<OutputDevice> {
    let parametrit = OutputDeviceParameters {
        sample_rate: NAYTETAAJUUS,
        channels_count: 1,
        channel_sample_count: AANIPUSKURI,
    };
    run_output_device(parametrit, move |ulos| match mikseri.lock() {
        Ok(mut mikseri) => mikseri.tayta(ulos),
        Err(_) => ulos.fill(0.0),
    })
    .ok()
}

/// Epäonnistunut tallennus ei saa kaataa peliä.
fn tallenna(polku: Option<&Path>, kaynnistin: &Kaynnistin) {
    if let Some(polku) = polku
        && let Err(virhe) = tallennus::kirjoita_tiedosto(polku, &kaynnistin.tallennus)
    {
        eprintln!("{}: {virhe}", polku.display());
    }
}

/// Välittää pyynnöt verkkosäikeelle ja ottaa sen vastaukset. Ilman verkkoa pyynnöt vain unohtuvat.
fn vaihda_netin_kanssa(kaynnistin: &mut Kaynnistin, netti: Option<&Netti>) {
    let pyynnot = take(&mut kaynnistin.pyynnot);
    if let Some(netti) = netti {
        for pyynto in pyynnot {
            netti.laheta(pyynto);
        }
        while let Ok(vastaus) = netti.vastaukset.try_recv() {
            kaynnistin.vastaanota(vastaus);
        }
    }
}

fn pelisilmukka(
    mut kaynnistin: Kaynnistin,
    ruutu: &mut [u32],
    kuva: &mut [u32],
    polku: Option<&Path>,
    netti: Option<Netti>,
) -> Result<(), minifb::Error> {
    let asetukset = WindowOptions {
        resize: true,
        scale: Scale::FitScreen,
        ..WindowOptions::default()
    };
    let mut otsikko = kaynnistin.otsikko();
    let mut ikkuna = Window::new(otsikko, IKKUNAN_POHJA.0, IKKUNAN_POHJA.1, asetukset)?;
    // Oletuspaikassa ikkuna voisi jatkua ruudun yli, ja sulkemisnappi jäisi piiloon.
    ikkuna.set_position(0, 0);
    ikkuna.set_target_fps(paivitystaajuus(&kaynnistin));
    let mut ikkunakuva = Ikkunakuva::default();

    let mikseri = Arc::new(Mutex::new(Mikseri::new()));
    // Laitetta ei suljeta itse: tinyaudion Drop voi kaataa pelin lopetuksessa, ja
    // käyttöjärjestelmä vapauttaa laitteen prosessin päättyessä joka tapauksessa.
    let _aanilaite = ManuallyDrop::new(avaa_aanilaite(Arc::clone(&mikseri)));

    let mut edellinen = Instant::now();
    let mut hiiri = (-1, -1);
    let mut nappi_alhaalla = false;
    let mut valikon_tila = None;

    while ikkuna.is_open() && !kaynnistin.lopeta {
        for nappain in ikkuna.get_keys_pressed(KeyRepeat::Yes) {
            if let Some(syote) = nappain_syotteeksi(nappain) {
                // toisto vain nuolille, ettei pohjassa pidetty välilyönti ammu sarjaa
                let toistuva = matches!(syote, Syote::Nuoli(..)) && kaynnistin.nuolet_toistuvat();
                if toistuva || ikkuna.is_key_pressed(nappain, KeyRepeat::No) {
                    kaynnistin.kasittele(syote);
                }
            }
        }

        // Pienennetyssä ikkunassa koko voi olla nolla.
        let (ikkuna_l, ikkuna_k) = ikkuna.get_size();
        let (ikkuna_l, ikkuna_k) = (ikkuna_l.max(1), ikkuna_k.max(1));
        let sovitus = Sovitus::new(ikkuna_l, ikkuna_k);

        let hiiren_paikka = ikkuna
            .get_unscaled_mouse_pos(MouseMode::Discard)
            .and_then(|(x, y)| sovitus.ruudulle(x as i32, y as i32));
        if let Some(piste) = hiiren_paikka {
            let nakyma = kaynnistin.nakyman_piste(piste.0, piste.1);
            if piste != hiiri {
                hiiri = piste;
                kaynnistin.kasittele(Syote::Hiiri(nakyma.0, nakyma.1));
            }
            let alhaalla = ikkuna.get_mouse_down(MouseButton::Left);
            if alhaalla && !nappi_alhaalla {
                kaynnistin.kasittele(Syote::Klikkaus(nakyma.0, nakyma.1));
            } else if !alhaalla && nappi_alhaalla {
                kaynnistin.kasittele(Syote::Vapautus(nakyma.0, nakyma.1));
            }
            nappi_alhaalla = alhaalla;
        } else {
            nappi_alhaalla = false;
        }

        kaynnistin.aseta_ohjaus(
            ikkuna.is_key_down(Key::Left),
            ikkuna.is_key_down(Key::Right),
            ikkuna.is_key_down(Key::Up),
        );
        if !ikkuna.is_active() {
            kaynnistin.keskeyta();
        }

        if let Ok(mut mikseri) = mikseri.lock() {
            if take(&mut kaynnistin.hiljenna) {
                mikseri.aseta_paalle(false);
            }
            mikseri.aseta_paalle(kaynnistin.aanet_paalla);
            for (aani, viive_ms) in kaynnistin.ota_aanet() {
                mikseri.soita(aani, viive_ms);
            }
        }

        vaihda_netin_kanssa(&mut kaynnistin, netti.as_ref());

        if take(&mut kaynnistin.tallenna) {
            tallenna(polku, &kaynnistin);
        }

        if kaynnistin.otsikko() != otsikko {
            otsikko = kaynnistin.otsikko();
            ikkuna.set_title(otsikko);
            ikkuna.set_target_fps(paivitystaajuus(&kaynnistin));
        }

        // Millisekunnin osat jäävät seuraavaan kierrokseen, jottei peli hidastu 60 Hz:llä.
        let kulunut = edellinen.elapsed().as_millis() as u32;
        kaynnistin.edista(kulunut);
        edellinen += Duration::from_millis(kulunut as u64);

        // Valikkoa ei piirretä, kun sen tila ja ikkunan koko ovat ennallaan.
        let tila = kaynnistin
            .valikon_avain(ikkuna.is_active())
            .map(|avain| (avain, ikkuna_l, ikkuna_k));
        let ennallaan = tila.is_some() && tila == valikon_tila;
        valikon_tila = tila;
        if !ennallaan {
            piirto::piirra(&kaynnistin, ruutu, kuva);
        }
        // Windows piirtää peitosta paljastuneen ikkunan itse edellisestä puskurista, joten
        // muuttumatonta kuvaa ei tarvitse lähettää uudelleen.
        if !ennallaan && ikkunakuva.sovita(ruutu, ikkuna_l, ikkuna_k, sovitus) {
            ikkuna.update_with_buffer(&ikkunakuva.pikselit, ikkuna_l, ikkuna_k)?;
        } else {
            ikkuna.update();
        }
    }
    // Ikkunan voi sulkea kesken pelin.
    kaynnistin.viimeistele();
    tallenna(polku, &kaynnistin);
    // Lähetys ei välttämättä ehdi perille, mutta tulos on jonossa seuraavaa käynnistystä varten.
    vaihda_netin_kanssa(&mut kaynnistin, netti.as_ref());
    Ok(())
}

fn main() -> ExitCode {
    let mut ruutu = vec![0u32; RUUDUN_LEVEYS * RUUDUN_KORKEUS];
    let mut kuva = vec![0u32; LEVEYS * KORKEUS];
    if let Some(polku) = kieli::oletuspolku() {
        kieli::lue_tiedosto(&polku);
    }

    #[cfg(debug_assertions)]
    let (kaynnistin, polku, netti) = {
        let liput = match kehitys::lue_liput() {
            Ok(liput) => liput,
            Err(virhe) => {
                eprintln!("{virhe}");
                return ExitCode::FAILURE;
            }
        };
        let mut kaynnistin = Kaynnistin::new(liput.siemen.unwrap_or_else(aikasiemen));
        // Kehitysajo ei koske pelaajan tallennukseen ilman lippua.
        let polku = liput.tallennus.as_ref().map(PathBuf::from);
        if let Some(polku) = &polku {
            kaynnistin.lataa(tallennus::lue_tiedosto(polku));
        }
        if let Some(kieli) = liput.kieli {
            kieli::aseta(kieli);
        }
        if let Some(i) = liput.valittu {
            kaynnistin.valitse(i);
        }
        if let Some(laji) = liput.laji() {
            kaynnistin.kaynnista(laji);
        }
        match &mut kaynnistin.peli {
            Some(Kaynnissa::Laivanupotus(peli)) => {
                if let Some(vaikeus) = liput.vaikeus {
                    peli.vaikeus = vaikeus;
                }
                if !liput.siirrot.is_empty() {
                    peli.kasittele(Syote::Ammu);
                }
                for ruutu in liput.siirrot {
                    if peli.vaihe == Vaihe::Ammunta {
                        peli.ammu(ruutu);
                    }
                }
            }
            Some(Kaynnissa::Luvut(peli)) => {
                if liput.lauta.is_some() || !liput.liu_ut.is_empty() {
                    peli.kasittele(Syote::Ammu);
                }
                if let Some(ruudut) = liput.lauta {
                    peli.lauta = Lukulauta::ruuduista(ruudut);
                }
                for suunta in liput.liu_ut {
                    if peli.vaihe == luvut::Vaihe::Pelissa {
                        peli.siirra(suunta);
                    }
                }
            }
            Some(Kaynnissa::Pasianssi(peli)) => {
                if let Some(vaikeus) = liput.vaikeus {
                    let taso = pasianssi::Vaikeus::KAIKKI
                        .into_iter()
                        .find(|taso| taso.nimi() == vaikeus.nimi());
                    peli.vaikeus = taso.unwrap_or(peli.vaikeus);
                }
                if liput.nostot.is_some() || liput.voitto || liput.jumissa {
                    peli.kasittele(Syote::Ammu);
                }
                for _ in 0..liput.nostot.unwrap_or(0) {
                    peli.kasittele(Syote::Kirjain('N'));
                }
                if liput.jumissa {
                    // Viimeinen siirto kerää ässän, ja peli toteaa lopun itse.
                    let tila = &mut peli.lauta.tila;
                    tila.pakka.clear();
                    tila.jate.clear();
                    tila.sarakkeet = Default::default();
                    tila.sarakkeet[0].kortit = vec![Kortti {
                        maa: Maa::Pata,
                        arvo: 9,
                    }];
                    tila.sarakkeet[1].kortit = vec![
                        Kortti {
                            maa: Maa::Risti,
                            arvo: 5,
                        },
                        Kortti {
                            maa: Maa::Hertta,
                            arvo: 1,
                        },
                    ];
                    peli.kasittele(Syote::Kirjain('A'));
                }
                if liput.voitto {
                    for (pino, &maa) in peli.lauta.tila.perustat.iter_mut().zip(&Maa::KAIKKI) {
                        *pino = (1..=KUNINGAS).map(|arvo| Kortti { maa, arvo }).collect();
                    }
                    peli.lauta.tila.sarakkeet = Default::default();
                    peli.lauta.tila.pakka.clear();
                    peli.lauta.tila.jate.clear();
                    peli.vaihe = pasianssi_peli::Vaihe::Voitto;
                }
            }
            Some(Kaynnissa::Tiilirikko(peli)) => {
                if let Some(kentta) = liput.kentta {
                    peli.kasittele(Syote::Ammu);
                    peli.lauta = Tiilipeli::new(kentta);
                    peli.kasittele(Syote::Ammu);
                    // Aikahyppy on rajattu, joten kuvakaappauksen hetki ajetaan pieninä askelina.
                    for _ in 0..liput.aika_ms / 16 {
                        peli.edista(16);
                        peli.kasittele(Syote::Ammu);
                    }
                }
            }
            Some(Kaynnissa::Asteroidit(peli)) => {
                if let Some(aalto) = liput.aalto {
                    peli.kasittele(Syote::Ammu);
                    peli.lauta = Avaruus::new(aalto, &mut Arpa::new(liput.siemen.unwrap_or(1)));
                    // Aikahyppy on rajattu, joten kuvakaappauksen hetki ajetaan pieninä askelina.
                    for _ in 0..liput.aika_ms / 16 {
                        peli.edista(16);
                    }
                }
            }
            Some(Kaynnissa::Silmukka(peli)) => {
                if let Some(itikoita) = liput.rengas {
                    peli.kasittele(Syote::Ammu);
                    // Pää sulkee renkaan kolmen askeleen päästä, ja häntä jää sen ulkopuolelle.
                    let mut ruudut: Vec<(i32, i32)> = (1..=18).map(|x| (x, 4)).collect();
                    ruudut.extend((5..=16).map(|y| (18, y)));
                    ruudut.extend((5..=17).rev().map(|x| (x, 16)));
                    ruudut.extend((7..=15).rev().map(|y| (5, y)));
                    peli.lauta.jalki = ruudut[0];
                    peli.lauta.mato = ruudut.into_iter().rev().collect();
                    peli.lauta.suunta = Suunta::Ylos;
                    let mato = peli.lauta.mato.clone();
                    peli.lauta.herkut.retain(|h| !mato.contains(&h.paikka));
                    peli.lauta.otukset = (0..itikoita)
                        .map(|i| Otus::uusi(Laji::Itikka, (7 + i * 2, 7 + i % 3 * 3), (1, 1), 0))
                        .collect();
                    // Aikahyppy on rajattu, joten kuvakaappauksen hetki ajetaan pieninä askelina.
                    for _ in 0..liput.aika_ms / 16 {
                        peli.edista(16);
                    }
                }
            }
            None => {}
        }
        kaynnistin.ota_aanet();
        if let Some(polku) = liput.kuvakaappaus {
            // Renkaan tehosteet on ajettu jo, eikä niitä saa hypätä yli.
            if liput.rengas.is_none() {
                kaynnistin.edista(liput.aika_ms);
            }
            piirto::piirra(&kaynnistin, &mut ruutu, &mut kuva);
            return match kehitys::tallenna_png(&polku, &ruutu, RUUDUN_LEVEYS, RUUDUN_KORKEUS) {
                Ok(()) => ExitCode::SUCCESS,
                Err(virhe) => {
                    eprintln!("{polku}: {virhe}");
                    ExitCode::FAILURE
                }
            };
        }
        // Kehitysajo ei koske oikeaan nettilistaan ilman lippua.
        let netti = liput.netti.as_deref().and_then(Netti::kaynnista);
        (kaynnistin, polku, netti)
    };
    #[cfg(not(debug_assertions))]
    let (kaynnistin, polku, netti) = {
        let mut kaynnistin = Kaynnistin::new(aikasiemen());
        let polku = tallennus::oletuspolku();
        if let Some(polku) = &polku {
            kaynnistin.lataa(tallennus::lue_tiedosto(polku));
        }
        (
            kaynnistin,
            polku,
            Netti::kaynnista(kaynnistin::netti::OSOITE),
        )
    };

    match pelisilmukka(kaynnistin, &mut ruutu, &mut kuva, polku.as_deref(), netti) {
        Ok(()) => ExitCode::SUCCESS,
        Err(virhe) => {
            eprintln!("{virhe}");
            ExitCode::FAILURE
        }
    }
}
