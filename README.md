# statusbar

lightweight and programmable statusbar for use with dwm

the statusbar is configured through the `make_statusbar!` macro in `src/config.rs`.
tasks are scheduled in a heap and are only updated when they are due.
