use image::GenericImageView;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Path to the input image (relative to workspace root)
  let input_path = "img/roles.jpg";

  // Load the image
  println!("Loading image from: {}", input_path);
  let img = image::open(input_path)?;

  let (img_width, img_height) = img.dimensions();
  println!("Image dimensions: {}x{}", img_width, img_height);

  // Calculate number of tiles
  let cols = 9;
  let rows = 3;

  // Grid dimensions
  let tile_width = img_width as f64 / cols as f64;
  let tile_height = img_height as f64 / rows as f64;
  println!("Tile dimensions: {}x{}", tile_width, tile_height);

  println!("Splitting into {}x{} grid ({} tiles total)", cols, rows, cols * rows);

  // Create output directory if it doesn't exist
  std::fs::create_dir_all("output")?;

  // Split the image into tiles
  for row in 0..rows {
    for col in 0..cols {
      let x = (col as f64 * tile_width).round() as u32;
      let y = (row as f64 * tile_height).round() as u32;

      // Calculate actual tile dimensions (may be smaller at edges)
      let actual_width = tile_width.floor() as u32;
      assert!(actual_width <= img_width.saturating_sub(x));
      let actual_height = tile_height.floor() as u32;
      assert!(actual_height <= img_height.saturating_sub(y));

      // Extract the tile
      let tile = img.view(x, y, actual_width, actual_height).to_image();

      // Save as PNG
      let output_path = format!("output/g{}{}.png", row + 1, col + 1);
      tile.save(&output_path)?;
      println!("Saved: {}", output_path);
    }
  }

  println!("Done! Split into {} tiles", cols * rows);
  Ok(())
}
