#![allow(dead_code)]
use macroquad::prelude::*;

/// A floating text particle
#[derive(Clone, Debug)]
pub struct FloatingText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: Color,
    pub life: f32,      // 0.0 to 1.0
    pub max_life: f32,
    pub velocity: Vec2,
    pub scale: f32,
}

impl FloatingText {
    pub fn new(text: &str, x: f32, y: f32, color: Color) -> Self {
        Self {
            text: text.to_string(),
            x,
            y,
            color,
            life: 1.0,
            max_life: 1.5, // Seconds
            velocity: Vec2::new(0.0, -30.0), // Float up
            scale: 1.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.life -= dt;
        let _t = 1.0 - (self.life / self.max_life);
        
        // Move
        self.x += self.velocity.x * dt;
        self.y += self.velocity.y * dt;
        
        // Fade out velocity
        self.velocity.y *= 0.95;
    }
    
    pub fn is_dead(&self) -> bool {
        self.life <= 0.0
    }
    
    pub fn draw(&self) {
        let alpha = (self.life / self.max_life).clamp(0.0, 1.0);
        
        let color = Color::new(self.color.r, self.color.g, self.color.b, alpha);
        
        // Outline for readability
        let outline_color = Color::new(0.0, 0.0, 0.0, alpha);
        draw_text(&self.text, self.x + 1.0, self.y + 1.0, 20.0 * self.scale, outline_color);
        
        draw_text(&self.text, self.x, self.y, 20.0 * self.scale, color);
    }
}

/// Simple interpolation helper
pub struct Tween {
    pub start: f32,
    pub end: f32,
    pub current: f32,
    pub speed: f32,
}

impl Tween {
    pub fn new(val: f32) -> Self {
        Self {
            start: val,
            end: val,
            current: val,
            speed: 10.0,
        }
    }
    
    pub fn target(&mut self, target: f32) {
        self.end = target;
    }
    
    pub fn update(&mut self, dt: f32) {
        let diff = self.end - self.current;
        if diff.abs() < 0.1 {
            self.current = self.end;
        } else {
            self.current += diff * self.speed * dt;
        }
    }
    
    pub fn value(&self) -> f32 {
        self.current
    }
}

impl Default for Tween {
    fn default() -> Self {
        Self::new(0.0)
    }
}
