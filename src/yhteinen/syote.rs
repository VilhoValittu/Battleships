/// Ikkunakirjastosta riippumaton syöte. Hiiren koordinaatit ovat kuvapuskurin pikseleitä.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syote {
    Hiiri(i32, i32),
    Klikkaus(i32, i32),
    Nuoli(i8, i8),
    /// Sarakekirjain A-J tai uusinnan K/E.
    Kirjain(char),
    /// Rivinäppäin, jossa 0 tarkoittaa riviä 10.
    Numero(u8),
    Ammu,
    Peru,
    VaihdaAanet,
}
