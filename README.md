# Built With Bevy Splashscreen

Provides an animated & customizable "Built with Bevy" splashscreen so you can show off your love for the engine in your own game!

https://github.com/user-attachments/assets/4af4ce64-f213-4a28-a354-6409b5ed2c56

# Usage

```rust
use bevy::prelude::*;
use built_with_bevy::{
    BevySplashscreenEnded, BevySplashscreenPlugin, SkipBevySplashscreen, StartBevySplashscreen,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevySplashscreenPlugin::default())
        .add_systems(Startup, splashscreen)
        .add_observer(ended)
        .run();
}

fn splashscreen(mut commands: Commands) {
    // trigger the splashscreen animation!
    commands.trigger(StartBevySplashscreen);
}

fn ended(_: On<BevySplashscreenEnded>) {
    // move onto main menu or other splashscreens, etc.
}
```
