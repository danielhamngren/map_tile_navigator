# Map Tile Navigator

This is a simple map visualization tool based on map tiles. The main focus is to be able to show map tiles served using the WMTS standard. For example styles created and served from [Mapbox](https://www.mapbox.com/).

## Running

```
cargo run -- --wmts_url [url to the capabilities-xml-document]
```

## Controls

Use the arrow keys for panning to the tile above, below, left or right.

Zoom in into specific Quadrant by using `W`, `Q`, `A` and `S`.

- Quadrant I: `W`
- Quadrant II: `Q`
- Quadrant III: `A`
- Quadrant IV: `S`

Zoom out with `space` or `-`.

Quit the program with `ESC`.
