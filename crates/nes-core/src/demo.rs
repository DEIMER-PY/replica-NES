//! ROM de demostración propia, generada en memoria.
//!
//! Permite probar el emulador de extremo a extremo (vídeo, audio y mandos) sin
//! depender de ninguna ROM externa. La usan los tests y la app como pantalla de prueba.

/// Tamaño de la cabecera iNES.
const HEADER_LEN: usize = 16;
/// Tamaño de un banco de PRG-ROM.
const PRG_LEN: usize = 16 * 1024;

/// Programa 6502 de la demo, ensamblado a mano y cargado en `$8000`.
///
/// Con el botón A del jugador 1 suelto la pantalla es azul y hay silencio; mientras
/// se mantiene pulsado la pantalla es roja y suena un tono de ~440 Hz.
#[rustfmt::skip]
const PROGRAM: &[u8] = &[
    // --- Arranque ---
    0x78,             // 8000  SEI            ; sin interrupciones IRQ
    0xD8,             // 8001  CLD
    0xA2, 0xFF,       // 8002  LDX #$FF
    0x9A,             // 8004  TXS            ; pila en $01FF
    0x2C, 0x02, 0x20, // 8005  BIT $2002      ; limpia el flag de vblank
    0x2C, 0x02, 0x20, // 8008  BIT $2002      ; espera dos vblank: la PPU ya está lista
    0x10, 0xFB,       // 800B  BPL $8008
    0x2C, 0x02, 0x20, // 800D  BIT $2002
    0x10, 0xFB,       // 8010  BPL $800D
    // --- Canal de pulso 1: tono continuo con volumen 0 ---
    0xA9, 0xB0,       // 8012  LDA #$B0       ; duty 50 %, volumen constante 0
    0x8D, 0x00, 0x40, // 8014  STA $4000
    0xA9, 0x01,       // 8017  LDA #$01
    0x8D, 0x15, 0x40, // 8019  STA $4015      ; activa el canal
    0xA9, 0xFD,       // 801C  LDA #$FD       ; periodo $0FD ≈ 440 Hz
    0x8D, 0x02, 0x40, // 801E  STA $4002
    0xA9, 0x00,       // 8021  LDA #$00
    0x8D, 0x03, 0x40, // 8023  STA $4003
    // --- Bucle principal: una vuelta por frame ---
    0x2C, 0x02, 0x20, // 8026  BIT $2002      ; espera al vblank
    0x10, 0xFB,       // 8029  BPL $8026
    0xA9, 0x01,       // 802B  LDA #$01       ; strobe de los mandos
    0x8D, 0x16, 0x40, // 802D  STA $4016
    0xA9, 0x00,       // 8030  LDA #$00
    0x8D, 0x16, 0x40, // 8032  STA $4016
    0xAD, 0x16, 0x40, // 8035  LDA $4016      ; primer bit leído: botón A
    0x29, 0x01,       // 8038  AND #$01
    0xF0, 0x06,       // 803A  BEQ $8042
    0xA2, 0x16,       // 803C  LDX #$16       ; A pulsado: rojo
    0xA0, 0xBF,       // 803E  LDY #$BF       ;            volumen 15
    0xD0, 0x04,       // 8040  BNE $8046
    0xA2, 0x21,       // 8042  LDX #$21       ; A suelto: azul
    0xA0, 0xB0,       // 8044  LDY #$B0       ;           volumen 0
    0x8C, 0x00, 0x40, // 8046  STY $4000
    0xA9, 0x3F,       // 8049  LDA #$3F       ; dirección de PPU $3F00: color de fondo
    0x8D, 0x06, 0x20, // 804B  STA $2006
    0xA9, 0x00,       // 804E  LDA #$00
    0x8D, 0x06, 0x20, // 8050  STA $2006
    0x8E, 0x07, 0x20, // 8053  STX $2007
    0x8D, 0x06, 0x20, // 8056  STA $2006      ; deja la dirección en $0000
    0x8D, 0x06, 0x20, // 8059  STA $2006
    0x4C, 0x26, 0x80, // 805C  JMP $8026
    // --- Interrupciones sin uso ---
    0x40,             // 805F  RTI
];

/// Dirección de la rutina de interrupción vacía (`RTI`).
const RTI_ADDR: u16 = 0x805F;
/// Dirección de arranque.
const RESET_ADDR: u16 = 0x8000;

/// Devuelve una ROM iNES mínima (mapper 0, 16 KB de PRG, CHR-RAM) escrita para este proyecto.
///
/// Con el botón A del jugador 1 suelto la pantalla es azul y hay silencio; mientras
/// se mantiene pulsado la pantalla es roja y suena un tono.
#[must_use]
pub fn demo_rom() -> Vec<u8> {
    let mut rom = vec![0u8; HEADER_LEN + PRG_LEN];

    // Cabecera: firma, 1 banco de PRG, 0 de CHR (CHR-RAM), mapper 0.
    rom[..4].copy_from_slice(b"NES\x1A");
    rom[4] = 1;

    let prg = &mut rom[HEADER_LEN..];
    prg[..PROGRAM.len()].copy_from_slice(PROGRAM);

    // Vectores al final del banco, que se ve reflejado en $C000-$FFFF.
    let vectors = [RTI_ADDR, RESET_ADDR, RTI_ADDR]; // NMI, RESET, IRQ
    for (i, addr) in vectors.into_iter().enumerate() {
        let offset = PRG_LEN - 6 + i * 2;
        prg[offset..offset + 2].copy_from_slice(&addr.to_le_bytes());
    }

    rom
}
