# Klassikot8000

**Suomeksi** | [English](README.en.md)

Tylsää? Kuusi vanhan liiton peliä yhdessä pienessä ikkunassa, C64- ja Amiga-henkisessä asussa. Ei tiliä, ei mainoksia, ei opeteltavaa. Avaa, valitse peli ja anna ajan kulua.

Pyörii vaikka perunalla.

## Pelit

### Laivanupotus
Sinä vastaan tietokone. Kummallakin on viisi laivaa, ja ammutte vuorotellen, kunnes toisen laivasto makaa pohjassa. Laivat eivät kosketa toisiaan edes kulmittain, joten uponneen laivan ympärys merkitään laudalle. Kumpi purjehtii kotiin perheensä luo?

*Hiiri tai nuolet ja välilyönti.*

### 2048
Työnnä lukuja laudalla, yhdistä samat ja tavoittele 2048:aa. Vielä yksi siirto, ja sitten lopetat. Ehkä. Kesken jäänyt peli odottaa seuraavaa kertaa.

*Nuolet tai hiiri.*

### Pasianssi
Se tuttu Klondike, jonka kanssa on kulutettu tuhansia työpäiviä. Kolme vaikeustasoa: helpolla nostat kortin kerrallaan ja saat perua, vaikealla nostat kolme, pakan saa käydä läpi kolmesti eikä mitään voi perua.

*Hiiri tai nuolet ja välilyönti.*

### Laatan murskain
Pidä pallo kentällä ja riko tiilet. Viisi kenttää, kolme palloa, ja nopeasta läpäisystä saa lisäpisteitä. Valkoinen tiili tuo toisen pallon ja kasvattaa mailaa, ja niin kauan kuin molemmat pallot ovat kentällä, tiilistä saa tuplapisteet. Kahden pallon vahtiminen on juuri niin vaikeaa kuin miltä se kuulostaa.

*Hiiri tai nuolet, laukaisu välilyönnillä tai klikkauksella.*

### Asteroidit
Yksinäinen alus, loputtomasti kiviä. Iso kivi hajoaa pienemmiksi, lentävät lautaset ampuvat takaisin, ja kun tilanne käy mahdottomaksi, voit aina hypätä sokkona jonnekin muualle. Aluksia on kolme. 10 000 pistettä tuo uuden, ja jokainen seuraava maksaa edellistä enemmän.

*Nuolet tai hiiri, hiiren oikea kiihdyttää, välilyönti ampuu, H hyppää.*

### Mato8000
Matopeli, jossa itseensä törmääminen on koko idea. Osuma sulkee silmukan, ja kaikki sen sisään jäänyt on saalista. Silmukka kuluttaa kuitenkin madon pituutta, joten omenoita kannattaa syödä välillä. Mitä enemmän saat kerralla, sitä moninkertaisemmat pisteet. Varo seiniä ja itikoita.

*Nuolet, WASD tai hiiri.*

## Hyvä tietää

- `Esc` palaa pelistä valikkoon, ja kesken jäänyt hyvä tulos pääsee silti listalle. `P` pysäyttää toimintapelit, `M` vaimentaa äänet ja `L` vaihtaa valikossa kielen suomen ja englannin välillä.
- Laivanupotusta ja Pasianssia lukuun ottamatta peleissä on Top 3 -lista myös netissä. Ilman nettiä pelaat ihan yhtä hyvin.
- Valikko näyttää pelin version ja kertoo, jos GitHubissa on uudempi julkaisu. Peli katsoo asian käynnistyessään.

## Pelaamaan

Hae omalle koneellesi sopiva paketti [julkaisuista](https://github.com/VilhoValittu/Klassikot8000/releases/latest) ja pura se. Lataus on Windowsille alle megan ja Linuxille alle 3 Mt.

- **Windows** (`windows-x64`): käynnistä `klassikot8000.exe`.
- **Linux** (`linux-x64`, `linux-arm64`): käynnistä `klassikot8000`. Peli tarvitsee X11:n tai XWaylandin ja äänille ALSA-kirjaston, jotka työpöytäkoneissa on yleensä valmiina.
- **macOS** (`macos-universal`, Intel ja Apple Silicon): käynnistä `Klassikot8000.app`. Paketilla ei ole Applen allekirjoitusta, joten macOS estää ensimmäisen käynnistyksen. Poista esto Päätteessä: `xattr -d com.apple.quarantine Klassikot8000.app`.
