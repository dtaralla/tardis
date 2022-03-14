use bevy::prelude::*;
use tardis::viewer::ViewerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ViewerPlugin {
            tle_source_url: "https://celestrak.com/NORAD/elements/gp.php?GROUP=starlink&FORMAT=tle"
                .to_string(),
            // For other constellations, visit for ex. https://celestrak.com/NORAD/elements/
        })
        .run();
}
