// Pelin suurin tulos. Tiilirikossa tiilistä saa enintään 8770 ja aikabonuksista 15000 pistettä.
// 2048:n teoreettinen huippu on 3 932 100 pistettä. Asteroidit rajaa tuloksen itse lukuun 999 999.
export const PELIT = { luvut: 4000000, tiilirikko: 30000, asteroidit: 1000000 };

const RIVI = /^([1-9][0-9]{0,8}) ([A-ZÄÖ0-9]{1,8})$/u;

export function lue_tulos(teksti, suurin) {
  const osat = RIVI.exec(teksti.trim());
  if (!osat) {
    return null;
  }
  const pisteet = Number(osat[1]);
  return pisteet <= suurin ? { pisteet, nimi: osat[2] } : null;
}
