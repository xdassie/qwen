use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct FrameCount(pub u64);

/// Trait for recording screenshots, always active and writes to /tmp each frame
pub trait Recorder {
    fn limit_reached(&self, frame_count: u64) -> bool;
    fn capture_screenshot(&self, frame_count: u64, commands: Commands, window_query: Query<Entity, With<Window>>);
    fn get_screenshot_limit(&self) -> u64;
}

#[derive(Resource, Clone)]
pub struct ScreenshotRecorder {
    screenshot_limit: u64,
}

impl ScreenshotRecorder {
    pub fn new(limit: u64) -> Self {
        Self { screenshot_limit: limit }
    }
}

impl Recorder for ScreenshotRecorder {
    fn limit_reached(&self, frame_count: u64) -> bool {
        frame_count >= self.screenshot_limit
    }
    
    fn capture_screenshot(
        &self,
        frame_count: u64,
        mut commands: Commands,
        window_query: Query<Entity, With<Window>>,
    ) {
        println!("DEBUG: Capturing screenshot {}", frame_count);
        
        // Spawn screenshot capture on the first window
        for _window_entity in window_query.iter() {
            let filename = format!("/tmp/screenshot_{}.png", frame_count);
            commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window()).observe(bevy::render::view::screenshot::save_to_disk(filename));
        }
    }
    
    fn get_screenshot_limit(&self) -> u64 {
        self.screenshot_limit
    }
}

