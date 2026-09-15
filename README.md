# Built With Bevy Splashscreen

Provides an animated & customizable "Built with Bevy" splashscreen so you can show off your love for the engine in your own game!

![Built with Bevy splashscreen](https://raw.githubusercontent.com/Aceeri/built_with_bevy/main/assets/splash.gif)

# Usage

```rust
use bevy::prelude::*;
use built_with_bevy::{BevySplashscreenEnded, BevySplashscreenPlugin, StartBevySplashscreen};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevySplashscreenPlugin)
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
