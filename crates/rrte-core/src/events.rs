use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// System event types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemEvent {
    /// Window events
    WindowResized { width: u32, height: u32 },
    WindowClosed,
    WindowFocused,
    WindowUnfocused,
    
    /// Input events
    KeyPressed { key: String, modifiers: KeyModifiers },
    KeyReleased { key: String, modifiers: KeyModifiers },
    MousePressed { button: MouseButton, x: f32, y: f32 },
    MouseReleased { button: MouseButton, x: f32, y: f32 },
    MouseMoved { x: f32, y: f32, delta_x: f32, delta_y: f32 },
    MouseWheelScrolled { delta_x: f32, delta_y: f32 },
    
    /// Engine events
    SceneChanged,
    CameraChanged,
    
    /// Custom events
    Custom { name: String, data: String },
}

/// Key modifier flags
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

impl Default for KeyModifiers {
    fn default() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            logo: false,
        }
    }
}

/// Mouse button types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// Event listener trait
pub trait EventListener {
    fn handle_event(&mut self, event: &SystemEvent) -> bool;
}

/// Event system for managing and dispatching events
pub struct Events {
    event_queue: VecDeque<SystemEvent>,
    listeners: Vec<Box<dyn EventListener>>,
    max_events: usize,
}

impl Events {
    /// Create a new event system
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            listeners: Vec::new(),
            max_events: 1000,
        }
    }

    /// Add an event to the queue
    pub fn push_event(&mut self, event: SystemEvent) {
        if self.event_queue.len() >= self.max_events {
            self.event_queue.pop_front(); // Remove oldest event
        }
        self.event_queue.push_back(event);
    }

    /// Poll and process all events in the queue
    pub fn poll(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            self.dispatch_event(&event);
        }
    }

    /// Dispatch an event to all listeners
    fn dispatch_event(&mut self, event: &SystemEvent) {
        for listener in &mut self.listeners {
            if listener.handle_event(event) {
                break; // Event was consumed
            }
        }
    }

    /// Add an event listener
    pub fn add_listener(&mut self, listener: Box<dyn EventListener>) {
        self.listeners.push(listener);
    }

    /// Remove all listeners
    pub fn clear_listeners(&mut self) {
        self.listeners.clear();
    }

    /// Check if there are pending events
    pub fn has_pending_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Get the number of pending events
    pub fn pending_event_count(&self) -> usize {
        self.event_queue.len()
    }

    /// Set the maximum number of events to keep in the queue
    pub fn set_max_events(&mut self, max_events: usize) {
        self.max_events = max_events;
        
        // Trim queue if it's too large
        while self.event_queue.len() > max_events {
            self.event_queue.pop_front();
        }
    }

    /// Clear all pending events
    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Events {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Events")
            .field("pending_events", &self.event_queue.len())
            .field("listeners", &self.listeners.len())
            .field("max_events", &self.max_events)
            .finish()
    }
}

/// Helper function to create common events
impl SystemEvent {
    /// Create a window resize event
    pub fn window_resize(width: u32, height: u32) -> Self {
        SystemEvent::WindowResized { width, height }
    }

    /// Create a key press event
    pub fn key_press(key: impl Into<String>, modifiers: KeyModifiers) -> Self {
        SystemEvent::KeyPressed {
            key: key.into(),
            modifiers,
        }
    }

    /// Create a key release event
    pub fn key_release(key: impl Into<String>, modifiers: KeyModifiers) -> Self {
        SystemEvent::KeyReleased {
            key: key.into(),
            modifiers,
        }
    }

    /// Create a mouse press event
    pub fn mouse_press(button: MouseButton, x: f32, y: f32) -> Self {
        SystemEvent::MousePressed { button, x, y }
    }

    /// Create a mouse release event
    pub fn mouse_release(button: MouseButton, x: f32, y: f32) -> Self {
        SystemEvent::MouseReleased { button, x, y }
    }

    /// Create a mouse move event
    pub fn mouse_move(x: f32, y: f32, delta_x: f32, delta_y: f32) -> Self {
        SystemEvent::MouseMoved { x, y, delta_x, delta_y }
    }

    /// Create a custom event
    pub fn custom(name: impl Into<String>, data: impl Into<String>) -> Self {
        SystemEvent::Custom {
            name: name.into(),
            data: data.into(),
        }
    }
}
