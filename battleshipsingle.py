from random import choice, randint

# Laivanupotuksen säännöt: 10x10 ruudukko ja viiden laivan laivasto (nimi, pituus).
KOKO = 10
SARAKKEET = "ABCDEFGHIJ"
RIVIT = [str(numero) for numero in range(1, KOKO + 1)]
LAIVAT = [
    ("Lentotukialus", 5),
    ("Taistelulaiva", 4),
    ("Hävittäjä", 3),
    ("Sukellusvene", 3),
    ("Partioalus", 2),
]


# ASETETAAN LAIVAT LAUDALLE
def sijoita_laivat():
    # Palauttaa listan (nimi, ruudut). Ruudut ovat joukko (rivi, sarake) -pareja.
    laivat = []
    varatut = set()
    for nimi, pituus in LAIVAT:
        # arvotaan paikkaa kunnes laiva mahtuu laudalle eikä osu toisen laivan päälle
        while True:
            # laivat ovat vaaka- tai pystysuunnassa, ei vinoon
            drivi, dsarake = choice([(0, 1), (1, 0)])
            rivi = randint(0, KOKO - 1 - drivi * (pituus - 1))
            sarake = randint(0, KOKO - 1 - dsarake * (pituus - 1))
            ruudut = {(rivi + drivi * i, sarake + dsarake * i) for i in range(pituus)}
            if not ruudut & varatut:
                break
        varatut |= ruudut
        laivat.append((nimi, ruudut))
    return laivat


# TULOSTETAAN PELILAUTA
def pelilauta(laivat, ammutut, nayta_laivat):
    # nayta_laivat: omalla laudalla näkyvät laivat (#), vastustajan laudalla ei.
    laivaruudut = set().union(*(ruudut for _, ruudut in laivat))
    print("   " + "".join(f" {kirjain}  " for kirjain in SARAKKEET))
    print(" " + "-" * (3 + 4 * KOKO - 1))
    for rivi in range(KOKO):
        solut = ""
        for sarake in range(KOKO):
            ruutu = (rivi, sarake)
            if ruutu in ammutut:
                merkki = "X" if ruutu in laivaruudut else "-"
            elif nayta_laivat and ruutu in laivaruudut:
                merkki = "#"
            else:
                merkki = " "
            solut += f" {merkki} |"
        print(f"{rivi + 1:2}|{solut}")
    print()


def tulosta_ruudut(
    pelaajan_laivat, vastustajan_laivat, pelaajan_ammutut, vastustajan_ammutut
):
    print("     \n         LAIVAN UPOTUS")
    print("       *****************")
    print("VASTUSTAJAN LAUTA (ammuntasi):")
    pelilauta(vastustajan_laivat, pelaajan_ammutut, False)
    print("OMA LAUTA (# = laiva, X = osuma, - = ohi):")
    pelilauta(pelaajan_laivat, vastustajan_ammutut, True)


def ruudun_nimi(ruutu):
    rivi, sarake = ruutu
    return f"{SARAKKEET[sarake]}{rivi + 1}"


# AMMUNTA. KYSYTÄÄN PELAAJALTA RUUTU, JOHON EI OLE AMMUTTU JO AIEMMIN.
def laukaus(ammutut):
    while True:
        # Vertailu listaan, koska "" ja "12" kuuluisivat merkkijonoon "123456789".
        rivi = input(f"Anna rivi numero 1-{KOKO}: ").strip()
        while rivi not in RIVIT:
            print("Syötä validi numero!")
            rivi = input(f"Anna rivi numero 1-{KOKO}: ").strip()
        sarake = input(f"Anna sarake kirjain A-{SARAKKEET[-1]}: ").strip().upper()
        while len(sarake) != 1 or sarake not in SARAKKEET:
            print("Syötä validi kirjain!")
            sarake = input(f"Anna sarake kirjain A-{SARAKKEET[-1]}: ").strip().upper()
        # Rivi -1, koska halutaan aloittaa numerosta 1 eikä numerosta 0
        ruutu = (int(rivi) - 1, SARAKKEET.index(sarake))
        if ruutu not in ammutut:
            return ruutu
        print("Ammuit jo tähän!")


# VASTUSTAJA. AMPUU SATUNNAISEEN RUUTUUN, JOHON EI OLE AMMUTTU AIEMMIN.
def vastustajan_laukaus(ammutut):
    vapaat = [(r, s) for r in range(KOKO) for s in range(KOKO) if (r, s) not in ammutut]
    return choice(vapaat)


# KÄSITELLÄÄN LAUKAUS. PALAUTETAAN (OSUMA, UPOTETUN LAIVAN NIMI TAI None).
def kasittele_laukaus(laivat, ammutut, ruutu):
    ammutut.add(ruutu)
    for nimi, ruudut in laivat:
        if ruutu in ruudut:
            # laiva uppoaa kun kaikkiin sen ruutuihin on osunut
            return True, (nimi if ruudut <= ammutut else None)
    return False, None


def kaikki_upotettu(laivat, ammutut):
    return all(ruudut <= ammutut for _, ruudut in laivat)


def tulos_teksti(ampuja, ruutu, osuma, upotettu):
    teksti = f"{ampuja} {ruudun_nimi(ruutu)}: "
    if upotettu:
        return teksti + f"OSUMA! {upotettu} upposi!"
    return teksti + ("OSUMA!" if osuma else "ohi.")


def pelaa():
    print("\n\n---------------------------------------------------------------")
    print("Tervetuloa pelaamaan laivanupotusta.")
    print("Ohjeet: ")
    print(
        f"Kummallakin on {KOKO}x{KOKO} ruudukossa 5 laivaa (pituudet 5, 4, 3, 3 ja 2)."
    )
    print(
        "Ammutte vuorotellen. Voittaja on se, joka upottaa ensin vastustajan kaikki laivat."
    )
    print("---------------------------------------------------------------")

    pelaajan_laivat = sijoita_laivat()
    vastustajan_laivat = sijoita_laivat()
    # ammutut ruudut tallennetaan sen laudan mukaan, johon on ammuttu
    pelaajan_ammutut = set()  # pelaajan ampumat ruudut vastustajan laudalla
    vastustajan_ammutut = set()  # vastustajan ampumat ruudut pelaajan laudalla
    viesti = ""

    while True:
        tulosta_ruudut(
            pelaajan_laivat, vastustajan_laivat, pelaajan_ammutut, vastustajan_ammutut
        )
        # edellisen kierroksen tulokset laudan alle
        print(viesti)
        ruutu = laukaus(pelaajan_ammutut)
        # tyhjennetään näyttö ja vierityshistoria, jotta näkyy vain yksi pelilauta
        print("\033[H\033[2J\033[3J", end="")
        osuma, upotettu = kasittele_laukaus(vastustajan_laivat, pelaajan_ammutut, ruutu)
        viesti = "\n" + tulos_teksti("Sinä ammuit", ruutu, osuma, upotettu)
        if kaikki_upotettu(vastustajan_laivat, pelaajan_ammutut):
            viesti += "\n\n****HIHIHIHIII VOITIT PELIN!****\n"
            break

        ruutu = vastustajan_laukaus(vastustajan_ammutut)
        osuma, upotettu = kasittele_laukaus(pelaajan_laivat, vastustajan_ammutut, ruutu)
        viesti += "\n" + tulos_teksti("Vastustaja ampui", ruutu, osuma, upotettu)
        if kaikki_upotettu(pelaajan_laivat, vastustajan_ammutut):
            viesti += "\n\nHÄVISIT PELIN!\n"
            break

    # pelin lopussa näytetään vielä viimeiset laudat ja lopputulos
    tulosta_ruudut(
        pelaajan_laivat, vastustajan_laivat, pelaajan_ammutut, vastustajan_ammutut
    )
    print(viesti)


if __name__ == "__main__":
    STARTTI = True
    while STARTTI:
        pelaa()
        uusinta = input("Haluatko pelata uudestaan? k = kyllä, e = ei : ")
        if uusinta != "k":
            STARTTI = False
            print("Lopetetaan peli.")
