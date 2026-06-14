//! Responsabilité : état clavier courant (touches enfoncées) interrogé par le joueur.

use std::collections::HashSet;

use winit::keyboard::KeyCode;

/// Ensemble des touches actuellement enfoncées.
#[derive(Default)]
pub struct InputState {
    pressed: HashSet<KeyCode>,
}

impl InputState {
    pub fn set(&mut self, key: KeyCode, down: bool) {
        if down {
            self.pressed.insert(key);
        } else {
            self.pressed.remove(&key);
        }
    }

    /// `true` si la touche est enfoncée.
    pub fn is(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Vide l'état (à l'ouverture d'un menu/chat pour éviter les touches collées).
    pub fn clear(&mut self) {
        self.pressed.clear();
    }
}
