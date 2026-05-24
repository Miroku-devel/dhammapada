<div align="center">
  <img src="assets/icon.png" width="120">

  <h1 style="font-size: 2em; margin: 10px 0;">Dhammapada</h1>
  <h3 style="font-size: 1em;">A multilingual Dhammapada reader for GNU/Linux.</h3>
  <img src="assets/screenshot.png" width="500">
</div>

## Language selection
The application will automatically detect your operating system's language upon startup.\
To manually override the language (for example, to run the app in German):

```
$ env LANGUAGE=de ./Dhammapada-*-x86_64.AppImage
```

## Building from source
To compile and run the project locally, ensure you have\
**Rust**, **GTK4**, and **Libadwaita** development libraries installed:\
\
Example for Ubuntu/Debian:

```bash
$ sudo apt install libgtk-4-dev libadwaita-1-dev
$ cargo run
```

## Copyright and License
The Rust application logic, UI design, assets and build scripts are licensed under the\
**GNU General Public License 3.0**.

All translations are the property of their respective authors or original publishers.\
For the complete list of translators, sources, and individual license terms for each language\
please consult the [CREDITS](./CREDITS) file.