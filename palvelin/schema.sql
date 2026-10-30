CREATE TABLE IF NOT EXISTS tulokset (
  peli TEXT NOT NULL,
  nimi TEXT NOT NULL,
  pisteet INTEGER NOT NULL,
  aika INTEGER NOT NULL,
  PRIMARY KEY (peli, nimi, pisteet)
);
CREATE INDEX IF NOT EXISTS tulokset_jarjestys ON tulokset (peli, pisteet DESC, aika ASC);
