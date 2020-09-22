// use elementtree::Element;
use ::image as image_image;
use bytes;
use reqwest;
use roxmltree;
use std::collections::HashMap;
use std::error::Error;
use structopt::StructOpt;

use gfx_device_gl::{CommandBuffer, Factory, Resources};
use piston_window::*;

#[derive(Debug)]
struct TileMatrix {
    identifier: String,
    scale_denominator: f64,
    top_left_corner: [f64; 2],
    tile_width: u32,
    tile_height: u32,
    matrix_width: u32,
    matrix_height: u32,
}

impl TileMatrix {
    fn from_node(tile_matrix_node: roxmltree::Node) -> TileMatrix {
        // TODO: Return Result and throw some errors!

        let mut identifier = String::from("");
        let mut scale_denominator = 0.0;
        let mut top_left_corner = [0.0, 0.0];
        let mut tile_width = 0;
        let mut tile_height = 0;
        let mut matrix_width = 0;
        let mut matrix_height = 0;

        for node in tile_matrix_node.children() {
            if node.is_element() {
                let node_text;
                match node.text() {
                    Some(value) => node_text = value,
                    None => continue,
                }
                match node.tag_name().name() {
                    "Identifier" => identifier = String::from(node_text),
                    "ScaleDenominator" => scale_denominator = node_text.parse::<f64>().unwrap(),
                    "TileWidth" => tile_width = node_text.parse::<u32>().unwrap(),
                    "TileHeight" => tile_height = node_text.parse::<u32>().unwrap(),
                    "MatrixWidth" => matrix_width = node_text.parse::<u32>().unwrap(),
                    "MatrixHeight" => matrix_height = node_text.parse::<u32>().unwrap(),
                    "TopLeftCorner" => {
                        let strings: Vec<&str> = node_text.split(' ').collect();
                        let floats: Vec<f64> = strings
                            .into_iter()
                            .map(|x| x.parse::<f64>().unwrap())
                            .collect();
                        for i in 0..floats.len() {
                            top_left_corner[i] = floats[i];
                        }
                    }
                    _ => println!("Unexpected node name!"), // TODO: Throw error here
                }
            }
        }

        TileMatrix {
            identifier,
            scale_denominator,
            top_left_corner,
            tile_width,
            tile_height,
            matrix_width,
            matrix_height,
        }
    }
}

struct ResourceURL {
    template: String,
}

impl ResourceURL {
    fn get_tile_url(&self, matrix_id: &str, column: u32, row: u32) -> String {
        let str_with_matrix_id = self.template.replace("{TileMatrix}", matrix_id);
        let str_with_column = str_with_matrix_id.replace("{TileCol}", &column.to_string());
        let complete_url = str_with_column.replace("{TileRow}", &row.to_string());

        complete_url
    }
}

// TODO: Check with a tile matrix if a certain tile is valid
struct TileManager {
    tile_texture: Option<G2dTexture>,
    current_row: u32,
    current_column: u32,
    current_matrix_id: String,
    resource_url: ResourceURL,
    texture_context: TextureContext<Factory, Resources, CommandBuffer>,
    texture_settings: TextureSettings,
}

impl TileManager {
    fn zoom_in(&mut self, quadrant: Quadrant) {
        self.next_matrix(); //

        //
        match quadrant {
            Quadrant::I => {
                self.current_row = 2 * self.current_row;
                self.current_column = 2 * self.current_column + 1;
            }
            Quadrant::II => {
                self.current_row = 2 * self.current_row;
                self.current_column = 2 * self.current_column;
            }
            Quadrant::III => {
                self.current_row = 2 * self.current_row + 1;
                self.current_column = 2 * self.current_column;
            }
            Quadrant::VI => {
                self.current_row = 2 * self.current_row + 1;
                self.current_column = 2 * self.current_column + 1;
            }
        }

        self.update_texture();
    }

    fn travel(&mut self, movement: Movement) {
        match movement {
            Movement::Up => self.current_row -= 1,
            Movement::Down => self.current_row += 1,
            Movement::Left => self.current_column -= 1,
            Movement::Right => self.current_column += 1,
        }

        self.update_texture();
    }

    fn zoom_out(&mut self) {
        self.prev_matrix();

        self.current_column /= 2;
        self.current_row /= 2;

        self.update_texture();
    }

    fn next_matrix(&mut self) {
        self.current_matrix_id = (self.current_matrix_id.parse::<u32>().unwrap() + 1).to_string();
    }

    fn prev_matrix(&mut self) {
        self.current_matrix_id = (self.current_matrix_id.parse::<u32>().unwrap() - 1).to_string();
    }

    fn update_texture(&mut self) {
        let tile_url = self.resource_url.get_tile_url(
            &self.current_matrix_id,
            self.current_column,
            self.current_row,
        );

        let tile_bytes = fetch_tile(tile_url).unwrap();
        let tile_image = image_image::load_from_memory(&tile_bytes)
            .unwrap()
            .to_rgba();

        // This isn't really how I want to do it. There exists a method for the texture struct
        // which is .update(context, image) but I haven't been able to use that yet.
        let tile_texture: G2dTexture = Texture::from_image(
            &mut self.texture_context,
            &tile_image,
            &self.texture_settings,
        )
        .unwrap();

        self.tile_texture = Option::from(tile_texture);
    }
}

#[derive(Clone, Copy)]
enum Quadrant {
    I,
    II,
    III,
    VI,
}

enum Movement {
    Up,
    Down,
    Left,
    Right,
}

fn parse_wmts_xml(wmts_document: &str) -> (ResourceURL, HashMap<String, TileMatrix>) {
    let mut resource_url = ResourceURL {
        template: String::new(),
    };

    let doc = roxmltree::Document::parse(wmts_document).unwrap();

    let mut tile_matrix_map = HashMap::new();

    if let Some(node) = doc.root().first_child() {
        let content_node = node
            .children()
            .find(|n| n.tag_name().name() == "Contents")
            .unwrap();

        let resource_url_node = content_node
            .descendants()
            .find(|n| n.tag_name().name() == "ResourceURL")
            .unwrap();

        if let Some(url) = resource_url_node.attribute("template") {
            resource_url.template = String::from(url);
        }

        let tile_matrix_set_node = content_node
            .children()
            .find(|n| n.tag_name().name() == "TileMatrixSet")
            .unwrap();
        for item in tile_matrix_set_node.children() {
            if item.tag_name().name() == "TileMatrix" {
                let tm = TileMatrix::from_node(item);

                tile_matrix_map.insert(tm.identifier.clone(), tm);
            }
        }
    }
    (resource_url, tile_matrix_map)
}

fn fetch_tile(url: String) -> Result<bytes::Bytes, Box<dyn std::error::Error>> {
    let body = reqwest::blocking::get(&url)?;
    Ok(body.bytes()?)
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Map tile navigator")]
struct Opt {
    #[structopt(long, env = "WMTS_URL")]
    wmts_url: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let wmts_capabilities_xml = reqwest::blocking::get(&opt.wmts_url)?;

    let (resource_url, tile_matrix_map) = parse_wmts_xml(&wmts_capabilities_xml.text()?);

    let tile_width = tile_matrix_map.get("0").unwrap().tile_width;
    let tile_height = tile_matrix_map.get("0").unwrap().tile_height;

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("Map tile navigator", [tile_width, tile_height])
            .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();

    let context = window.create_texture_context();

    window.set_lazy(true);

    let mut tm = TileManager {
        tile_texture: None,
        current_row: 0,
        current_column: 0,
        current_matrix_id: String::from("0"),
        resource_url: resource_url,
        texture_context: context,
        texture_settings: TextureSettings::new(),
    };

    tm.update_texture();
    let line = Line::new([0.5, 0.5, 0.5, 0.5], 0.5);
    let mut focus_rect = None;

    while let Some(e) = window.next() {
        if let Some(Button::Keyboard(key)) = e.release_args() {
            match key {
                Key::W => tm.zoom_in(Quadrant::I),
                Key::Q => tm.zoom_in(Quadrant::II),
                Key::A => tm.zoom_in(Quadrant::III),
                Key::S => tm.zoom_in(Quadrant::VI),
                Key::R => tm.zoom_in(Quadrant::VI), // For colemak users
                Key::Up => tm.travel(Movement::Up),
                Key::Down => tm.travel(Movement::Down),
                Key::Right => tm.travel(Movement::Right),
                Key::Left => tm.travel(Movement::Left),
                Key::Space => tm.zoom_out(),
                Key::Minus => tm.zoom_out(),
                _ => {}
            }
            focus_rect = None;
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::W => focus_rect = Some(Quadrant::I),
                Key::Q => focus_rect = Some(Quadrant::II),
                Key::A => focus_rect = Some(Quadrant::III),
                Key::S => focus_rect = Some(Quadrant::VI),
                Key::R => focus_rect = Some(Quadrant::VI), // For colemak users
                _ => {}
            }
        }
        window.draw_2d(&e, |c, g, _| {
            clear([1.0; 4], g);
            if let Some(texture) = &tm.tile_texture {
                image(texture, c.transform, g);
            }
            line.draw(
                [
                    tile_width as f64 / 2.0,
                    0.0,
                    tile_width as f64 / 2.0,
                    tile_height as f64,
                ],
                &c.draw_state,
                c.transform,
                g,
            );
            line.draw(
                [
                    0.0,
                    tile_height as f64 / 2.0,
                    tile_width as f64,
                    tile_height as f64 / 2.0,
                ],
                &c.draw_state,
                c.transform,
                g,
            );

            if let Some(quadrant) = focus_rect {
                draw_focus_rect(tile_height as f64, tile_width as f64, quadrant, g, &c);
            }
        });
    }

    Ok(())
}

fn draw_focus_rect<G: Graphics>(
    tile_height: f64,
    tile_width: f64,
    quadrant: Quadrant,
    g: &mut G,
    c: &Context,
) {
    let rect: [f64; 4];
    match quadrant {
        Quadrant::I => rect = [tile_width / 2.0, 0.0, tile_width, tile_height / 2.0],
        Quadrant::II => rect = [0.0, 0.0, tile_width / 2.0, tile_height / 2.0],
        Quadrant::III => rect = [0.0, tile_height / 2.0, tile_width / 2.0, tile_height],
        Quadrant::VI => rect = [tile_width / 2.0, tile_height / 2.0, tile_width, tile_height],
    }

    rectangle([0.5, 0.5, 0.5, 0.5], rect, c.transform, g);
}
