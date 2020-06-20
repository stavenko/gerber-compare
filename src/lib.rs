extern crate png;
use png::OutputInfo;
use std::fs::{/*remove_file*/ File};
use std::io::prelude::*;
use std::path::{ Path};
use std::process::Command;
use std::env; 


struct Color {
  r: f32,
  g: f32,
  b: f32,
  a: f32
}

impl Color {
  pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
    Color {
      r: r as f32 / 255.0,
      b: b as f32 / 255.0,
      g: g as f32 / 255.0,
      a: a as f32 / 255.0
    }
  }

  pub fn is_exists(&self, threshold: f32) -> bool {
    return self.a > threshold || self.r > threshold || self.g > threshold || self.b > threshold;
  }
}

fn read_png(file: &Path) -> (Vec<u8>, OutputInfo) {
  let decoder = png::Decoder::new(File::open(file).unwrap());
  let (info, mut reader) = decoder.read_info().unwrap();
  let mut buf = vec![0; info.buffer_size()];
  // Read the next frame. Currently this function should only called once.
  // The default options
  // let info = reader.info();

  reader.next_frame(&mut buf).unwrap();
  (buf, info)
}


fn is_data_equal((buf1, info1): (Vec<u8>, OutputInfo), (buf2, info2): (Vec<u8>, OutputInfo)) -> bool {
  let color_type1 = info1.color_type;
  let color_type2 = info2.color_type;
  let mut total_pixels: f32 = 0.0;
  let mut equal_pixels: f32 = 0.0;



  println!("file1 color type {:?}", color_type1);
  println!("file2 color type {:?}", color_type2);

  let threshold = 0.01;


  for i in 0..512 {
    for j in 0..512 {
      let x1 = ((i as f32 / 512.0) * (info1.width as f32)).floor() as u32;
      let y1 = ((j as f32 / 512.0) * (info1.height as f32)).floor() as u32;

      let x2 = ((i as f32 / 512.0) * (info2.width as f32)).floor() as u32;
      let y2 = ((j as f32 / 512.0) * (info2.height as f32)).floor() as u32;

      let ix1: usize = (info1.width * y1 + x1) as usize;
      let ix2: usize = (info2.width * y2 + x2) as usize;

      let color1 = Color::new(buf1[ix1], buf1[ix1+1], buf1[ix1 + 2], buf1[ix1+3]);
      let color2 = Color::new(buf2[ix2], buf2[ix2+1], buf2[ix2 + 2], buf2[ix2+3]);
      total_pixels+=1.0;
      if color1.is_exists(threshold) == color2.is_exists(threshold) {
        equal_pixels += 1.0;
      }
    }
  };
  println!("result: {}/ {} = {}", equal_pixels, total_pixels, equal_pixels/ total_pixels);

  equal_pixels / total_pixels > 0.95
}

fn gerber_convert(grb: &Path, svg: &Path) {
  Command::new("gerbv")
    .arg(grb.to_str().unwrap())
    .arg("--export=svg")
    .arg("-o")
    .arg(svg.to_str().unwrap())
    .output().unwrap();

}

fn inkscape_convert(svg: &Path, png: &Path) {
  Command::new("inkscape")
    .arg("-d")
    .arg("150")
    .arg(svg.to_str().unwrap())
    .arg("-o")
    .arg(png.to_str().unwrap())
    .output()
    .unwrap();
}

fn save_my_svg(path: &Path, svg: String) {
  let mut file = File::create(path).unwrap();
  file.write_all(svg.as_bytes()).unwrap();
}


fn make_conversions(
  source_file_grb: &Path,
  gerb_view_result_svg: &Path,
  gerb_view_result_png: &Path,
  my_result_svg: &Path,
  my_result_png: &Path
  ) {
  gerber_convert(source_file_grb, gerb_view_result_svg);
  inkscape_convert(gerb_view_result_svg, gerb_view_result_png);
  inkscape_convert(my_result_svg, my_result_png);
}



pub fn svg_is_same(svg_source: String, path: &Path) -> bool {
  let expected_result = path.to_path_buf();
  let working_dir = env::current_dir().unwrap().join(expected_result.parent().unwrap()).canonicalize().unwrap();
  let source_file_grb = working_dir.join(expected_result.file_name().unwrap()); // grb
  let gerb_view_result_svg = source_file_grb.with_extension("svg"); // svg_
  let gerb_view_result_png = source_file_grb.with_extension("png"); // svg_
  let my_result_svg = gerb_view_result_png.with_file_name(
    format!("{}{}.svg", source_file_grb.file_stem().map(|s| s.to_str()).flatten().unwrap(), "_result")
  );
  let my_result_png = my_result_svg.with_extension("png");

  println!("{} \n {} \n {} \n {} \n {}",
           source_file_grb.display(),
           gerb_view_result_svg.display(),
           gerb_view_result_png.display(),
           my_result_svg.display(),
           my_result_png.display()
           );

  save_my_svg(&my_result_svg, svg_source);

  make_conversions(&source_file_grb, &gerb_view_result_svg, &gerb_view_result_png, &my_result_svg, &my_result_png);

  let result_content = read_png(&gerb_view_result_png);
  let expected_content = read_png(&my_result_png);

  /*

     for debug purpose leaving it as-is
  remove_file(&gerb_view_result_png);
  remove_file(&gerb_view_result_svg);
  remove_file(&my_result_png);
  remove_file(&my_result_svg);


  */
  
  is_data_equal(result_content, expected_content)
}


#[test]
fn main() {

  let svg = include_str!("/Users/vstavenko/projects/gerber-ruster/output.svg");
  let file = String::from("../gerber-ruster/test-visual/gerber/strokes/circle-tool-single-segment.gbr");
  let path = Path::new(&file);

  let is_equal = svg_is_same(String::from(svg), &path);
  println!("same file {:?}", is_equal);
  assert_eq!(is_equal, true);

}
