# Reference Rust: ADC / SBC decimal (8 et 16 bits)

STATUS: reviewed (Claude). Ces deux fonctions generalisent l'algorithme de `01_CPU_65816.md` section 6 a N quartets.
Verification faite hors depot sur les VRAIS tests de l'oracle (SingleStepTests 65816, fichiers 69 et E9, modes natif et emulation,
cas D=1): 0 ecart pour ADC et SBC. Le code Rust ci-dessous a ete compare a un modele Python sur 60 000 vecteurs: 0 ecart.

REGLES:
- Copier ces fonctions TELLES QUELLES (ne pas les reecrire, ne pas les "simplifier").
- ADC: remplace `adc_decimal_n` dans `alu/adc.rs`. L'appel 8 bits passe `nibbles = 2` (et non 1), l'appel 16 bits `nibbles = 4`.
  Les masques de resultat sont geres dans la fonction. Le wrapper `adc_decimal(cpu, m, wide)` garde le reste (A.h conserve en 8 bits, C, V, set_nz).
- SBC: remplace `sbc_decimal_n` dans `alu/sbc.rs` (meme signature `(a, m, carry_in, wide)`). `m` est passe NON inverse (la fonction fait `!m`).
- SBC: l'arithmetique est SIGNEE (`i32`) expres: apres la correction `-= 6 << sh`, un resultat negatif donne C = 0. Une version non signee (`wrapping_sub`) est FAUSSE (retenue C inversee pour des quartets non-BCD).
- Piege deja vu: additionner en binaire puis corriger chaque chiffre apres coup est FAUX (la retenue de chaque quartet doit
  etre propagee dans la boucle, avant de passer au quartet suivant). De meme, une soustraction chiffre par chiffre avec emprunt est FAUSSE.

```rust
// ADC decimal, n nibbles (2 = 8 bit, 4 = 16 bit). Returns (result, carry, overflow).
fn adc_decimal_n(a: u32, m: u32, carry_in: bool, nibbles: usize) -> (u32, bool, bool) {
    let mut r: u32 = 0;
    let mut c: u32 = carry_in as u32;
    let mut v = false;
    for i in 0..nibbles {
        let sh = 4 * i as u32;
        let low = (1u32 << sh) - 1;
        let mask = 0xFu32 << sh;
        r = (a & mask) + (m & mask) + (c << sh) + (r & low);
        if i == nibbles - 1 {
            v = (!(a ^ m) & (a ^ r) & (0x8u32 << sh)) != 0;
        }
        if r > ((9u32 << sh) | low) {
            r += 6u32 << sh;
        }
        c = (r > ((1u32 << (sh + 4)) - 1)) as u32;
    }
    (r & ((1u32 << (4 * nibbles as u32)) - 1), c != 0, v)
}

// SBC decimal: same loop with m' = !m, in SIGNED arithmetic (a negative intermediate
// result after the correction means "no carry"). Returns (result, carry, overflow).
fn sbc_decimal_n(a: u32, m: u32, carry_in: bool, wide: bool) -> (u32, bool, bool) {
    let nibbles: usize = if wide { 4 } else { 2 };
    let a = a as i32;
    let mp = (!m) as i32;
    let mut r: i32 = 0;
    let mut c: i32 = carry_in as i32;
    let mut v = false;
    for i in 0..nibbles {
        let sh = 4 * i as u32;
        let low = (1i32 << sh) - 1;
        let mask = 0xFi32 << sh;
        r = (a & mask) + (mp & mask) + (c << sh) + (r & low);
        if i == nibbles - 1 {
            v = (!(a ^ mp) & (a ^ r) & (0x8i32 << sh)) != 0;
        }
        if r <= (1i32 << (sh + 4)) - 1 {
            r -= 6i32 << sh;
        }
        c = (r > (1i32 << (sh + 4)) - 1) as i32;
    }
    ((r as u32) & ((1u32 << (4 * nibbles as u32)) - 1), c != 0, v)
}
```
