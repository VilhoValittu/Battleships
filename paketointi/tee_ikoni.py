"""Piirtää ohjelman logon ja kirjoittaa siitä ikonitiedostot tähän kansioon.

Ajo: python paketointi/tee_ikoni.py

Tulokset ovat versionhallinnassa, joten kääntäminen ei tarvitse Pythonia. Ei riippuvuuksia,
jotta logoa voi muuttaa millä tahansa koneella. Ajo kestää noin 20 sekuntia.
"""

import struct
import zlib
from pathlib import Path

KANSIO = Path(__file__).parent

# Samat värit kuin käynnistimen maisemassa, src/yhteinen/piirto.rs.
TAIVAS_YLA = (0x0A, 0x06, 0x26)
TAIVAS_ALA = (0xD8, 0x50, 0x8C)
MERI_YLA = (0x18, 0x20, 0x6E)
MERI_ALA = (0x05, 0x08, 0x1E)
MERI_VIIVA = (0x4F, 0xA8, 0xE0)
KELTAINEN = (0xFF, 0xE9, 0x6B)
ORANSSI = (0xFF, 0x88, 0x00)
LAIVA = (0x12, 0x0A, 0x38)
VAAHTO = (0xA8, 0xDC, 0xFA)
VALKOINEN = (0xFF, 0xFF, 0xFF)

HORISONTTI = 0.66
AURINKO = (0.5, 0.40, 0.31)
TAIVAAN_RAIDAT = 10
# Auringon alaosan raidat: (alku, korkeus). Raidat paksunevat horisonttia kohti.
AURINGON_RAIDAT = [(0.43, 0.012), (0.485, 0.018), (0.54, 0.024), (0.60, 0.030)]
PIENEN_RAIDAT = [(7 / 16, 1 / 16), (9 / 16, 1 / 16)]
TAHDET = [(0.13, 0.12), (0.26, 0.24), (0.86, 0.15), (0.74, 0.07), (0.92, 0.33), (0.08, 0.36)]

RUNKO = [
    (0.09, 0.610),
    (0.68, 0.610),
    (0.71, 0.575),
    (0.93, 0.575),
    (0.85, 0.705),
    (0.14, 0.705),
]
LAIVAN_OSAT = [
    RUNKO,
    # Takatykki
    [(0.13, 0.610), (0.14, 0.575), (0.21, 0.575), (0.22, 0.610)],
    [(0.04, 0.580), (0.15, 0.588), (0.15, 0.600), (0.04, 0.592)],
    # Savupiippu
    [(0.25, 0.610), (0.26, 0.505), (0.32, 0.505), (0.33, 0.610)],
    # Komentosilta
    [(0.34, 0.610), (0.34, 0.510), (0.40, 0.510), (0.40, 0.430), (0.56, 0.430), (0.60, 0.510), (0.64, 0.510), (0.64, 0.610)],
    # Masto ja sen poikkipuu
    [(0.462, 0.430), (0.472, 0.235), (0.484, 0.235), (0.494, 0.430)],
    [(0.42, 0.315), (0.54, 0.315), (0.54, 0.330), (0.42, 0.330)],
    # Etutykki
    [(0.72, 0.575), (0.73, 0.530), (0.80, 0.530), (0.81, 0.575)],
    [(0.78, 0.538), (0.93, 0.505), (0.935, 0.518), (0.78, 0.553)],
]
# Pienessä koossa ohuet osat katoaisivat, joten siluetti on karkeampi ja sen reunat osuvat
# 16 pikselin ruudukkoon: näin pienimmät koot pysyvät terävinä.
PIENI_LAIVA = [
    [(x / 16, y / 16) for x, y in osa]
    for osa in (
        [(1, 10), (11, 10), (11, 9), (15, 9), (14, 12), (2, 12)],
        [(3, 10), (3, 8), (5, 8), (5, 10)],
        [(6, 10), (6, 6), (11, 6), (11, 10)],
        [(8, 6), (8, 2), (9, 2), (9, 6)],
    )
]


def sekoita(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def rajat(monikulmio):
    xs, ys = zip(*monikulmio)
    return min(xs), min(ys), max(xs), max(ys)


def sisalla(x, y, monikulmio):
    sisa = False
    edellinen = monikulmio[-1]
    for piste in monikulmio:
        if (piste[1] > y) != (edellinen[1] > y):
            leikkaus = piste[0] + (y - piste[1]) * (edellinen[0] - piste[0]) / (edellinen[1] - piste[1])
            if x < leikkaus:
                sisa = not sisa
        edellinen = piste
    return sisa


class Kuva:
    """Logon yksi yksityiskohtataso: mitä pienempi ikoni, sitä vähemmän piirrettävää."""

    def __init__(self, koko):
        self.koko = koko
        self.pieni = koko <= 32
        self.reuna = 0.0 if koko <= 24 else 0.03
        self.pyoristys = 0.14 if koko <= 24 else 0.19
        self.raidat = PIENEN_RAIDAT if self.pieni else AURINGON_RAIDAT
        osat = PIENI_LAIVA if self.pieni else LAIVAN_OSAT
        self.laiva = [(rajat(osa), osa) for osa in osat]
        self.horisontti = 11 / 16 if self.pieni else HORISONTTI
        self.vesiraja = 12 / 16 if self.pieni else 0.705
        self.vaahto = VALKOINEN if self.pieni else VAAHTO
        # Viivan on oltava vähintään pikselin paksuinen, jotta se ei puuroudu taustaan.
        self.viiva = max(0.006, 0.9 / koko)
        if koko < 32:
            self.merilinjat = []
        elif self.pieni:
            self.merilinjat = [0.86]
        else:
            self.merilinjat = [0.745, 0.80, 0.885]
        self.sateet = [] if self.pieni else [-1.25, -0.72, -0.34, 0.34, 0.72, 1.25]

    def maski(self, x, y):
        a, b = self.reuna, 1.0 - self.reuna
        if not (a <= x <= b and a <= y <= b):
            return False
        r = self.pyoristys
        dx = max(a + r - x, x - (b - r), 0.0)
        dy = max(a + r - y, y - (b - r), 0.0)
        return dx * dx + dy * dy <= r * r

    def tausta(self, x, y):
        if y >= self.horisontti:
            return self.meri(x, y)
        kaista = int(y / self.horisontti * TAIVAAN_RAIDAT) / (TAIVAAN_RAIDAT - 1)
        taivas = sekoita(TAIVAS_YLA, TAIVAS_ALA, kaista)
        if not self.pieni:
            for tx, ty in TAHDET:
                if abs(x - tx) < 0.007 and abs(y - ty) < 0.007:
                    return VALKOINEN
        ax, ay, ar = AURINKO
        if (x - ax) ** 2 + (y - ay) ** 2 <= ar * ar:
            if any(alku <= y < alku + korkeus for alku, korkeus in self.raidat):
                return taivas
            t = (y - (ay - ar)) / (self.horisontti - (ay - ar))
            return sekoita(KELTAINEN, ORANSSI, min(1.0, t * 1.15))
        return taivas

    def meri(self, x, y):
        syvyys = (y - self.horisontti) / (1.0 - self.horisontti)
        meri = sekoita(MERI_YLA, MERI_ALA, syvyys)
        if y < self.horisontti + self.viiva:
            return MERI_VIIVA
        for linja in self.merilinjat:
            if abs(y - linja) < self.viiva / 2:
                return sekoita(meri, MERI_VIIVA, 0.85)
        for k in self.sateet:
            sade_x = 0.5 + k * syvyys
            if abs(x - sade_x) < self.viiva / 2:
                return sekoita(meri, MERI_VIIVA, 0.85)
        return meri

    def vari(self, x, y):
        # Vesiraja erottaa tumman rungon tummasta merestä.
        if self.vesiraja <= y < self.vesiraja + max(0.010, 1.0 / self.koko) and 0.0625 <= x <= 0.9375:
            return self.vaahto
        for (x0, y0, x1, y1), osa in self.laiva:
            if x0 <= x <= x1 and y0 <= y <= y1 and sisalla(x, y, osa):
                return LAIVA
        return self.tausta(x, y)

    def piirra(self):
        n = 8 if self.koko <= 64 else 4
        rivit = []
        for py in range(self.koko):
            rivi = []
            for px in range(self.koko):
                r = g = b = peitto = 0.0
                for sy in range(n):
                    y = (py + (sy + 0.5) / n) / self.koko
                    for sx in range(n):
                        x = (px + (sx + 0.5) / n) / self.koko
                        if self.maski(x, y):
                            c = self.vari(x, y)
                            r += c[0]
                            g += c[1]
                            b += c[2]
                            peitto += 1
                if peitto:
                    rivi.append((round(r / peitto), round(g / peitto), round(b / peitto), round(255 * peitto / (n * n))))
                else:
                    rivi.append((0, 0, 0, 0))
            rivit.append(rivi)
        return rivit


def png(rivit):
    def lohko(tyyppi, data):
        return struct.pack(">I", len(data)) + tyyppi + data + struct.pack(">I", zlib.crc32(tyyppi + data))

    korkeus, leveys = len(rivit), len(rivit[0])
    raaka = b"".join(b"\0" + bytes(k for p in rivi for k in p) for rivi in rivit)
    return (
        b"\x89PNG\r\n\x1a\n"
        + lohko(b"IHDR", struct.pack(">IIBBBBB", leveys, korkeus, 8, 6, 0, 0, 0))
        + lohko(b"IDAT", zlib.compress(raaka, 9))
        + lohko(b"IEND", b"")
    )


def dib(rivit):
    """Ikonin pieni koko bittikarttana, jonka Windowsin vanhatkin osat osaavat."""
    koko = len(rivit)
    otsake = struct.pack("<IiiHHIIiiII", 40, koko, koko * 2, 1, 32, 0, 0, 0, 0, 0, 0)
    kuva = b"".join(bytes((p[2], p[1], p[0], p[3])) for rivi in reversed(rivit) for p in rivi)
    # Alfa ratkaisee läpinäkyvyyden, mutta tyhjä AND-maski kuuluu silti muotoon.
    maskin_rivi = ((koko + 31) // 32) * 4
    return otsake + kuva + bytes(maskin_rivi * koko)


def ico(kuvat):
    osat = [png(rivit) if koko >= 256 else dib(rivit) for koko, rivit in kuvat]
    tulos = struct.pack("<HHH", 0, 1, len(osat))
    paikka = 6 + 16 * len(osat)
    for (koko, _), data in zip(kuvat, osat):
        tulos += struct.pack("<BBBBHHII", koko % 256, koko % 256, 0, 0, 1, 32, len(data), paikka)
        paikka += len(data)
    return tulos + b"".join(osat)


def icns(kuvat):
    data = b"".join(tyyppi + struct.pack(">I", len(p) + 8) + p for tyyppi, p in kuvat)
    return b"icns" + struct.pack(">I", len(data) + 8) + data


def argb(rivit):
    """Raaka kuva Linuxin ikkunalle: pikselit u32-lukuina pienin tavu ensin."""
    return b"".join(struct.pack("<I", p[3] << 24 | p[0] << 16 | p[1] << 8 | p[2]) for rivi in rivit for p in rivi)


def main():
    kuvat = {koko: Kuva(koko).piirra() for koko in (16, 24, 32, 48, 64, 128, 256, 512)}
    (KANSIO / "ikoni.ico").write_bytes(ico([(k, kuvat[k]) for k in (16, 24, 32, 48, 64, 256)]))
    (KANSIO / "ikoni.icns").write_bytes(
        icns([(b"ic07", png(kuvat[128])), (b"ic08", png(kuvat[256])), (b"ic09", png(kuvat[512]))])
    )
    (KANSIO / "ikoni.png").write_bytes(png(kuvat[256]))
    (KANSIO / "ikoni48.argb").write_bytes(argb(kuvat[48]))


if __name__ == "__main__":
    main()
