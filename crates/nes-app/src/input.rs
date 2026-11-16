//! Controles: teclado y mandos convertidos en botones de la NES.

use eframe::egui::{self, Key};
use gilrs::{Axis, Button as PadButton, Gilrs};
use nes_core::{Button, Player};
use serde::{Deserialize, Serialize};

/// Los dos puertos de mando, en orden.
pub const PLAYERS: [Player; 2] = [Player::One, Player::Two];

/// Inclinación mínima del stick analógico para contar como dirección.
const STICK_THRESHOLD: f32 = 0.5;

/// Botones pulsados en un mando de NES.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PadState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
}

impl PadState {
    /// Estado de un botón.
    pub const fn get(&self, button: Button) -> bool {
        match button {
            Button::Up => self.up,
            Button::Down => self.down,
            Button::Left => self.left,
            Button::Right => self.right,
            Button::A => self.a,
            Button::B => self.b,
            Button::Select => self.select,
            Button::Start => self.start,
        }
    }

    /// Une dos fuentes: un botón está pulsado si lo está en cualquiera de ellas.
    pub const fn merge(self, other: Self) -> Self {
        Self {
            up: self.up || other.up,
            down: self.down || other.down,
            left: self.left || other.left,
            right: self.right || other.right,
            a: self.a || other.a,
            b: self.b || other.b,
            select: self.select || other.select,
            start: self.start || other.start,
        }
    }
}

/// Tecla asignada a cada botón de un mando.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyMap {
    pub up: Key,
    pub down: Key,
    pub left: Key,
    pub right: Key,
    pub a: Key,
    pub b: Key,
    pub select: Key,
    pub start: Key,
}

impl KeyMap {
    /// Teclas por defecto del jugador 1: flechas, `X`/`Z`, `Enter` y `Retroceso`.
    pub const PLAYER_ONE: Self = Self {
        up: Key::ArrowUp,
        down: Key::ArrowDown,
        left: Key::ArrowLeft,
        right: Key::ArrowRight,
        a: Key::X,
        b: Key::Z,
        select: Key::Backspace,
        start: Key::Enter,
    };

    /// Teclas por defecto del jugador 2: `I`/`J`/`K`/`L`, `H`/`G`, `Y` y `T`.
    pub const PLAYER_TWO: Self = Self {
        up: Key::I,
        down: Key::K,
        left: Key::J,
        right: Key::L,
        a: Key::H,
        b: Key::G,
        select: Key::T,
        start: Key::Y,
    };

    /// Tecla asignada a un botón.
    pub const fn key(&self, button: Button) -> Key {
        match button {
            Button::Up => self.up,
            Button::Down => self.down,
            Button::Left => self.left,
            Button::Right => self.right,
            Button::A => self.a,
            Button::B => self.b,
            Button::Select => self.select,
            Button::Start => self.start,
        }
    }

    /// Tecla asignada a un botón, para cambiarla.
    const fn key_mut(&mut self, button: Button) -> &mut Key {
        match button {
            Button::Up => &mut self.up,
            Button::Down => &mut self.down,
            Button::Left => &mut self.left,
            Button::Right => &mut self.right,
            Button::A => &mut self.a,
            Button::B => &mut self.b,
            Button::Select => &mut self.select,
            Button::Start => &mut self.start,
        }
    }

    /// Botones pulsados según `is_down`, que indica si una tecla está pulsada.
    fn state(&self, is_down: impl Fn(Key) -> bool) -> PadState {
        PadState {
            up: is_down(self.up),
            down: is_down(self.down),
            left: is_down(self.left),
            right: is_down(self.right),
            a: is_down(self.a),
            b: is_down(self.b),
            select: is_down(self.select),
            start: is_down(self.start),
        }
    }
}

/// Teclas de los dos jugadores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBindings {
    pub player_one: KeyMap,
    pub player_two: KeyMap,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            player_one: KeyMap::PLAYER_ONE,
            player_two: KeyMap::PLAYER_TWO,
        }
    }
}

impl KeyBindings {
    /// Teclas de un jugador.
    pub const fn map(&self, player: Player) -> &KeyMap {
        match player {
            Player::One => &self.player_one,
            Player::Two => &self.player_two,
        }
    }

    const fn map_mut(&mut self, player: Player) -> &mut KeyMap {
        match player {
            Player::One => &mut self.player_one,
            Player::Two => &mut self.player_two,
        }
    }

    /// Qué botón tiene asignada una tecla, si alguno.
    pub fn find(&self, key: Key) -> Option<(Player, Button)> {
        PLAYERS.into_iter().find_map(|player| {
            Button::ALL
                .into_iter()
                .find(|&button| self.map(player).key(button) == key)
                .map(|button| (player, button))
        })
    }

    /// Asigna `key` a un botón.
    ///
    /// Si la tecla ya estaba en otro botón, los dos intercambian sus teclas: así nunca
    /// queda una tecla en dos botones ni un botón sin tecla.
    pub fn rebind(&mut self, player: Player, button: Button, key: Key) {
        let previous = self.map(player).key(button);
        if let Some((other_player, other_button)) = self.find(key) {
            *self.map_mut(other_player).key_mut(other_button) = previous;
        }
        *self.map_mut(player).key_mut(button) = key;
    }

    /// Lee el teclado. Devuelve el estado de los dos mandos.
    ///
    /// Si la interfaz está usando el teclado (un campo de texto, un menú abierto), no
    /// se pulsa nada: así escribir o navegar por los menús no mueve al personaje.
    pub fn read(&self, ctx: &egui::Context) -> [PadState; 2] {
        if ctx.egui_wants_keyboard_input() {
            return [PadState::default(); 2];
        }
        ctx.input(|input| {
            // Con Ctrl/Alt pulsado es un atajo de la app (Ctrl+O...), no un botón.
            if input.modifiers.command || input.modifiers.alt {
                return [PadState::default(); 2];
            }
            PLAYERS.map(|player| self.map(player).state(|key| input.key_down(key)))
        })
    }
}

/// Mandos conectados (USB, Bluetooth).
///
/// El primer mando conectado controla al jugador 1 y el segundo al jugador 2.
pub struct Gamepads {
    /// `None` si el sistema no ofrece acceso a mandos; la app sigue con teclado.
    gilrs: Option<Gilrs>,
}

impl Default for Gamepads {
    fn default() -> Self {
        Self {
            gilrs: Gilrs::new().ok(),
        }
    }
}

impl Gamepads {
    /// Lee los mandos. Devuelve el estado de los dos jugadores.
    pub fn read(&mut self) -> [PadState; 2] {
        let mut pads = [PadState::default(); 2];
        let Some(gilrs) = &mut self.gilrs else {
            return pads;
        };
        // Hay que consumir los eventos para que gilrs actualice el estado de cada mando.
        while gilrs.next_event().is_some() {}

        for (pad, (_, gamepad)) in pads.iter_mut().zip(gilrs.gamepads()) {
            let (stick_left, stick_right) = stick_direction(gamepad.value(Axis::LeftStickX));
            // El eje Y de gilrs es positivo hacia arriba.
            let (stick_down, stick_up) = stick_direction(gamepad.value(Axis::LeftStickY));
            *pad = PadState {
                up: gamepad.is_pressed(PadButton::DPadUp) || stick_up,
                down: gamepad.is_pressed(PadButton::DPadDown) || stick_down,
                left: gamepad.is_pressed(PadButton::DPadLeft) || stick_left,
                right: gamepad.is_pressed(PadButton::DPadRight) || stick_right,
                // Botón inferior (A en Xbox, ✕ en PlayStation) = A; izquierdo (X, □) = B.
                a: gamepad.is_pressed(PadButton::South),
                b: gamepad.is_pressed(PadButton::West),
                select: gamepad.is_pressed(PadButton::Select),
                start: gamepad.is_pressed(PadButton::Start),
            };
        }
        pads
    }

    /// Nombres de los mandos conectados, en el orden de los jugadores.
    pub fn names(&self) -> Vec<String> {
        self.gilrs.as_ref().map_or_else(Vec::new, |gilrs| {
            gilrs
                .gamepads()
                .take(PLAYERS.len())
                .map(|(_, gamepad)| gamepad.name().to_owned())
                .collect()
        })
    }
}

/// Convierte un eje analógico en dos direcciones digitales: (negativa, positiva).
fn stick_direction(value: f32) -> (bool, bool) {
    (value < -STICK_THRESHOLD, value > STICK_THRESHOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keys_do_not_collide() {
        let bindings = KeyBindings::default();
        let mut keys: Vec<Key> = PLAYERS
            .iter()
            .flat_map(|&player| Button::ALL.map(|button| bindings.map(player).key(button)))
            .collect();
        let total = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), total, "hay una tecla asignada a dos botones");
    }

    #[test]
    fn key_map_reads_pressed_keys() {
        let state = KeyMap::PLAYER_ONE.state(|key| matches!(key, Key::X | Key::ArrowLeft));
        assert_eq!(
            state,
            PadState {
                a: true,
                left: true,
                ..PadState::default()
            }
        );
        for button in Button::ALL {
            let expected = matches!(button, Button::A | Button::Left);
            assert_eq!(state.get(button), expected, "{button:?}");
        }
    }

    #[test]
    fn merge_is_a_logical_or() {
        let keyboard = PadState {
            a: true,
            ..PadState::default()
        };
        let gamepad = PadState {
            start: true,
            ..PadState::default()
        };
        let merged = keyboard.merge(gamepad);
        assert!(merged.a && merged.start);
        assert!(!merged.b);
        assert_eq!(merged.merge(PadState::default()), merged);
    }

    #[test]
    fn rebind_assigns_a_free_key() {
        let mut bindings = KeyBindings::default();
        bindings.rebind(Player::One, Button::A, Key::Space);
        assert_eq!(bindings.map(Player::One).key(Button::A), Key::Space);
        assert_eq!(bindings.find(Key::X), None, "la tecla anterior queda libre");
    }

    #[test]
    fn rebind_swaps_with_the_button_that_had_the_key() {
        let mut bindings = KeyBindings::default();
        // `H` es el botón A del jugador 2; pasa a ser el A del jugador 1.
        bindings.rebind(Player::One, Button::A, Key::H);
        assert_eq!(bindings.find(Key::H), Some((Player::One, Button::A)));
        assert_eq!(bindings.find(Key::X), Some((Player::Two, Button::A)));
    }

    #[test]
    fn rebind_to_the_same_key_changes_nothing() {
        let mut bindings = KeyBindings::default();
        bindings.rebind(Player::One, Button::A, Key::X);
        assert_eq!(bindings, KeyBindings::default());
    }

    #[test]
    fn stick_needs_to_pass_the_threshold() {
        assert_eq!(stick_direction(0.0), (false, false));
        assert_eq!(stick_direction(0.3), (false, false));
        assert_eq!(stick_direction(0.9), (false, true));
        assert_eq!(stick_direction(-0.9), (true, false));
    }

    #[test]
    fn bindings_round_trip_through_ron() {
        let bindings = KeyBindings::default();
        let text = ron::to_string(&bindings).unwrap();
        assert_eq!(ron::from_str::<KeyBindings>(&text).unwrap(), bindings);
    }
}
