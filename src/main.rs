#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(debug_assertions)]
mod kehitys;
mod ui;

use std::mem::{ManuallyDrop, take};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use tinyaudio::{OutputDevice, OutputDeviceParameters, run_output_device};

use ui::aanet::{Mikseri, NAYTETAAJUUS};
#[cfg(debug_assertions)]
use ui::peli::Vaihe;
use ui::peli::{Peli, Syote};
use ui::piirto::{self, KORKEUS, LEVEYS};

const OTSIKKO: &str = "Laivanupotus";
const PAIVITYKSIA_SEKUNNISSA: usize = 30;
// Noin 36 ms: kytkin hiljentää äänet heti, mutta puskuri ei tyhjene kesken.
const AANIPUSKURI: usize = 400;

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

fn pelisilmukka(mut peli: Peli, kuva: &mut [u32]) -> Result<(), minifb::Error> {
    let asetukset = WindowOptions {
        scale: Scale::FitScreen,
        ..WindowOptions::default()
    };
    let mut ikkuna = Window::new(OTSIKKO, LEVEYS, KORKEUS, asetukset)?;
    ikkuna.set_target_fps(PAIVITYKSIA_SEKUNNISSA);

    let mikseri = Arc::new(Mutex::new(Mikseri::new()));
    // Laitetta ei suljeta itse: tinyaudion Drop voi kaataa pelin lopetuksessa, ja
    // käyttöjärjestelmä vapauttaa laitteen prosessin päättyessä joka tapauksessa.
    let _aanilaite = ManuallyDrop::new(avaa_aanilaite(Arc::clone(&mikseri)));

    let mut edellinen = Instant::now();
    let mut hiiri = (-1, -1);
    let mut nappi_alhaalla = false;

    while ikkuna.is_open() && !peli.lopeta {
        for nappain in ikkuna.get_keys_pressed(KeyRepeat::Yes) {
            if let Some(syote) = nappain_syotteeksi(nappain) {
                // toisto vain nuolille, ettei pohjassa pidetty välilyönti ammu sarjaa
                let toistuva = matches!(syote, Syote::Nuoli(..));
                if toistuva || ikkuna.is_key_pressed(nappain, KeyRepeat::No) {
                    peli.kasittele(syote);
                }
            }
        }

        if let Some((x, y)) = ikkuna.get_mouse_pos(MouseMode::Discard) {
            let piste = (x as i32, y as i32);
            if piste != hiiri {
                hiiri = piste;
                peli.kasittele(Syote::Hiiri(piste.0, piste.1));
            }
            let alhaalla = ikkuna.get_mouse_down(MouseButton::Left);
            if alhaalla && !nappi_alhaalla {
                peli.kasittele(Syote::Klikkaus(piste.0, piste.1));
            }
            nappi_alhaalla = alhaalla;
        } else {
            nappi_alhaalla = false;
        }

        if let Ok(mut mikseri) = mikseri.lock() {
            mikseri.aseta_paalle(peli.aanet_paalla);
            for (aani, viive_ms) in take(&mut peli.aanijono) {
                mikseri.soita(aani, viive_ms);
            }
        }

        let nyt = Instant::now();
        peli.edista(nyt.duration_since(edellinen).as_millis() as u32);
        edellinen = nyt;

        // Vesi ja tähtäin elävät koko ajan, joten joka päivitys piirretään kokonaan.
        piirto::piirra(&peli, kuva);
        ikkuna.update_with_buffer(kuva, LEVEYS, KORKEUS)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let mut kuva = vec![0u32; LEVEYS * KORKEUS];

    #[cfg(debug_assertions)]
    let peli = {
        let liput = match kehitys::lue_liput() {
            Ok(liput) => liput,
            Err(virhe) => {
                eprintln!("{virhe}");
                return ExitCode::FAILURE;
            }
        };
        let mut peli = Peli::new(liput.siemen.unwrap_or_else(aikasiemen));
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
        peli.aanijono.clear();
        if let Some(polku) = liput.kuvakaappaus {
            peli.edista(liput.aika_ms);
            piirto::piirra(&peli, &mut kuva);
            return match kehitys::tallenna_png(&polku, &kuva, LEVEYS, KORKEUS) {
                Ok(()) => ExitCode::SUCCESS,
                Err(virhe) => {
                    eprintln!("{polku}: {virhe}");
                    ExitCode::FAILURE
                }
            };
        }
        peli
    };
    #[cfg(not(debug_assertions))]
    let peli = Peli::new(aikasiemen());

    match pelisilmukka(peli, &mut kuva) {
        Ok(()) => ExitCode::SUCCESS,
        Err(virhe) => {
            eprintln!("{virhe}");
            ExitCode::FAILURE
        }
    }
}
