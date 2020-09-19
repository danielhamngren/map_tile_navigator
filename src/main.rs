// use elementtree::Element;
use roxmltree;
use std::collections::HashMap;
use std::fs;

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

fn parse_wmts_xml(path: &str) -> (String, HashMap<String, TileMatrix>) {
    let mut resource_url = String::new();

    let wmts_document = fs::read_to_string(path).unwrap();

    let doc = roxmltree::Document::parse(&wmts_document).unwrap();

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
            resource_url = String::from(url);
        }
        println!("tag name {:?}", resource_url);

        let tile_matrix_set_node = content_node
            .children()
            .find(|n| n.tag_name().name() == "TileMatrixSet")
            .unwrap();
        for item in tile_matrix_set_node.children() {
            if item.tag_name().name() == "TileMatrix" {
                println!("ITEM {:?}", item.tag_name().name());
                let tm = TileMatrix::from_node(item);
                println!("tilematrix: {:?}", &tm);

                tile_matrix_map.insert(tm.identifier.clone(), tm);
            }
        }
    }
    (resource_url, tile_matrix_map)
}

fn main() {
    let (resource_url, tile_matrix_map) = parse_wmts_xml("wmts.xml");
}
