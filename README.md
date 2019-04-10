# tidy
Minimalist X11 window manager, neat. 👌

**This project is a heavy work in progress!**

I've been bouncing around between different window managers and decided that none of them exactly suited my tastes.
The basic ideas behind tidy are:

* While I do things, I want them to take up as much screen space as possible.
* I want to reach things related to system settings in one place.
* I want easy and intuitive tiling across multiple monitors.
* Rounded borders.

The window manager will be as minimal as possible, just supporting fullscreen mode and a very simple tiled mode.
Configuration will probably never be external, since I'm recompiling it all the time anyways and rust macros make it
very easy to provide a nice API for configuration.

In addition to that, there will be several small applications (probably using GTK) that will offer a sort of
"Control center" which shows remaining charge, wifi connections, the time, and so on, while you hold down a configured
key; as well as a taskbar that will only appear in fullscreen mode.
