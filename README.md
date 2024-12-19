# LPG

LPG is a tool for generating posters and paintings for [Lethal Posters](https://thunderstore.io/c/lethal-company/p/femboytv/LethalPosters/) and [Lethal Paintings](https://thunderstore.io/c/lethal-company/p/femboytv/LethalPaintings/) by [femboy.tv](https://femboy.tv/).

## Usage

### Instructions

1. Donwload and extract the latest release.
2. Create a "./input" directory and add your images, or specify your own input directory (-i).
3. Verify the poster and painting template images are in "./templates", or specify the template directory (-t).
4. Run the executable (see below). Specify an output directory if desired (-o).
5. Move the BepInEx/plugins/LethalPosters and BepInEx/plugins/LethalPaintings directories to your mod's main directory.

The following is equivalent to running `./lpg.exe` with no options:

```sh
.\lpg.exe -t "./templates" -i "./input" -o "./output"
```

For Unix systems, use `./lpg` instead of `.\lpg.exe`. You know what you're doing. :wink:

### Manual

```txt
.\lpg.exe --help
A poster/painting generation tool for Lethal Posters and Lethal Paintings

Usage: lpg.exe [OPTIONS]

Options:
  -t, --templates <TEMPLATES>  The directory containing the poster and painting template images [default: ./templates]
  -i, --input <INPUT>          The directory containing the images to generate posters and paintings for [default: ./input]
  -o, --output <OUTPUT>        The directory containing the images to generate posters and paintings for [default: ./output]
  -h, --help                   Print help
  -V, --version                Print version
```

## Build

If you would like to build the tool yourself, simply download the source code and run `cargo build --release` from the extracted directory.
