Usage
-----

To have background music play when the game starts, place a file named `music.ogg` next to the game executable (same directory as the compiled binary). The code uses Raylib's music streaming API and expects an OGG file; other formats may work depending on raylib build support.

If `music.ogg` is missing or fails to load, the game will continue without music and log a warning to stderr.

Controls
- ENTER: confirm / restart
- Q: quit
- ESC: toggle mouse capture

Notes
- Audio is initialized when starting the game (after the main menu). Music playback is updated each frame while the game runs and cleaned up when quitting from a Game Over screen.
