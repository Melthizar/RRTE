use crate::{SystemEvent, KeyModifiers, MouseButton};
use std::collections::HashMap;
use rrte_math::Vec2;

/// Key state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Released,
    Pressed,
    JustPressed,
    JustReleased,
}

/// Input system for tracking keyboard and mouse state
#[derive(Debug)]
pub struct Input {
    // Keyboard state
    keys: HashMap<String, KeyState>,
    key_modifiers: KeyModifiers,
    
    // Mouse state
    mouse_buttons: HashMap<MouseButton, KeyState>,
    mouse_position: Vec2,
    last_mouse_position: Vec2,
    mouse_delta: Vec2,
    mouse_wheel_delta: Vec2,
    
    // Internal state
    just_pressed_keys: Vec<String>,
    just_released_keys: Vec<String>,
    just_pressed_mouse_buttons: Vec<MouseButton>,
    just_released_mouse_buttons: Vec<MouseButton>,
}

impl Input {
    /// Create a new input system
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            key_modifiers: KeyModifiers::default(),
            mouse_buttons: HashMap::new(),
            mouse_position: Vec2::ZERO,
            last_mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            mouse_wheel_delta: Vec2::ZERO,
            just_pressed_keys: Vec::new(),
            just_released_keys: Vec::new(),
            just_pressed_mouse_buttons: Vec::new(),
            just_released_mouse_buttons: Vec::new(),
        }
    }

    /// Update input state (call once per frame)
    pub fn update(&mut self) {
        // Update key states
        for key in &self.just_pressed_keys {
            if let Some(state) = self.keys.get_mut(key) {
                if *state == KeyState::JustPressed {
                    *state = KeyState::Pressed;
                }
            }
        }
        
        for key in &self.just_released_keys {
            if let Some(state) = self.keys.get_mut(key) {
                if *state == KeyState::JustReleased {
                    *state = KeyState::Released;
                }
            }
        }

        // Update mouse button states
        for button in &self.just_pressed_mouse_buttons {
            if let Some(state) = self.mouse_buttons.get_mut(button) {
                if *state == KeyState::JustPressed {
                    *state = KeyState::Pressed;
                }
            }
        }
        
        for button in &self.just_released_mouse_buttons {
            if let Some(state) = self.mouse_buttons.get_mut(button) {
                if *state == KeyState::JustReleased {
                    *state = KeyState::Released;
                }
            }
        }

        // Clear just pressed/released lists
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();

        // Reset mouse delta and wheel delta
        self.mouse_wheel_delta = Vec2::ZERO;
    }

    /// Handle system events
    pub fn handle_event(&mut self, event: &SystemEvent) {
        match event {
            SystemEvent::KeyPressed { key, modifiers } => {
                self.key_modifiers = modifiers.clone();
                self.keys.insert(key.clone(), KeyState::JustPressed);
                self.just_pressed_keys.push(key.clone());
            }
            SystemEvent::KeyReleased { key, modifiers } => {
                self.key_modifiers = modifiers.clone();
                self.keys.insert(key.clone(), KeyState::JustReleased);
                self.just_released_keys.push(key.clone());
            }
            SystemEvent::MousePressed { button, x, y } => {
                self.mouse_position = Vec2::new(*x, *y);
                self.mouse_buttons.insert(button.clone(), KeyState::JustPressed);
                self.just_pressed_mouse_buttons.push(button.clone());
            }
            SystemEvent::MouseReleased { button, x, y } => {
                self.mouse_position = Vec2::new(*x, *y);
                self.mouse_buttons.insert(button.clone(), KeyState::JustReleased);
                self.just_released_mouse_buttons.push(button.clone());
            }
            SystemEvent::MouseMoved { x, y, delta_x, delta_y } => {
                self.last_mouse_position = self.mouse_position;
                self.mouse_position = Vec2::new(*x, *y);
                self.mouse_delta = Vec2::new(*delta_x, *delta_y);
            }
            SystemEvent::MouseWheelScrolled { delta_x, delta_y } => {
                self.mouse_wheel_delta = Vec2::new(*delta_x, *delta_y);
            }
            _ => {}
        }
    }

    // Keyboard queries
    
    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: &str) -> bool {
        matches!(
            self.keys.get(key),
            Some(KeyState::Pressed) | Some(KeyState::JustPressed)
        )
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: &str) -> bool {
        matches!(self.keys.get(key), Some(KeyState::JustPressed))
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: &str) -> bool {
        matches!(self.keys.get(key), Some(KeyState::JustReleased))
    }

    /// Get the current key modifiers
    pub fn key_modifiers(&self) -> &KeyModifiers {
        &self.key_modifiers
    }

    // Mouse queries
    
    /// Check if a mouse button is currently pressed
    pub fn is_mouse_button_pressed(&self, button: &MouseButton) -> bool {
        matches!(
            self.mouse_buttons.get(button),
            Some(KeyState::Pressed) | Some(KeyState::JustPressed)
        )
    }

    /// Check if a mouse button was just pressed this frame
    pub fn is_mouse_button_just_pressed(&self, button: &MouseButton) -> bool {
        matches!(self.mouse_buttons.get(button), Some(KeyState::JustPressed))
    }

    /// Check if a mouse button was just released this frame
    pub fn is_mouse_button_just_released(&self, button: &MouseButton) -> bool {
        matches!(self.mouse_buttons.get(button), Some(KeyState::JustReleased))
    }

    /// Get the current mouse position
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Get the mouse movement delta since last frame
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Get the mouse wheel scroll delta for this frame
    pub fn mouse_wheel_delta(&self) -> Vec2 {
        self.mouse_wheel_delta
    }

    /// Get the previous mouse position
    pub fn last_mouse_position(&self) -> Vec2 {
        self.last_mouse_position
    }

    // Convenience methods for common keys/buttons
    
    /// Check if the left mouse button is pressed
    pub fn is_left_mouse_pressed(&self) -> bool {
        self.is_mouse_button_pressed(&MouseButton::Left)
    }

    /// Check if the right mouse button is pressed
    pub fn is_right_mouse_pressed(&self) -> bool {
        self.is_mouse_button_pressed(&MouseButton::Right)
    }

    /// Check if the middle mouse button is pressed
    pub fn is_middle_mouse_pressed(&self) -> bool {
        self.is_mouse_button_pressed(&MouseButton::Middle)
    }

    /// Check if shift is held
    pub fn is_shift_held(&self) -> bool {
        self.key_modifiers.shift
    }

    /// Check if ctrl is held
    pub fn is_ctrl_held(&self) -> bool {
        self.key_modifiers.ctrl
    }

    /// Check if alt is held
    pub fn is_alt_held(&self) -> bool {
        self.key_modifiers.alt
    }

    /// Reset all input state
    pub fn reset(&mut self) {
        self.keys.clear();
        self.mouse_buttons.clear();
        self.key_modifiers = KeyModifiers::default();
        self.mouse_position = Vec2::ZERO;
        self.last_mouse_position = Vec2::ZERO;
        self.mouse_delta = Vec2::ZERO;
        self.mouse_wheel_delta = Vec2::ZERO;
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

// Hash implementation for MouseButton
impl std::hash::Hash for MouseButton {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            MouseButton::Left => 0u8.hash(state),
            MouseButton::Right => 1u8.hash(state),
            MouseButton::Middle => 2u8.hash(state),
            MouseButton::Other(id) => {
                3u8.hash(state);
                id.hash(state);
            }
        }
    }
}

// Eq implementation for MouseButton
impl Eq for MouseButton {}
