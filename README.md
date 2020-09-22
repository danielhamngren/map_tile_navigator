# Map Tile Navigator

This is a simple map visualization tool based on map tiles. The main focus is to be able to show map tiles served using the WMTS standard. For example styles created and served from [Mapbox](https://www.mapbox.com/).

![map tile navigator demo](https://user-images.githubusercontent.com/5391285/93893459-f1a82a00-fced-11ea-9d9d-b41ec426ad71.gif)

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
