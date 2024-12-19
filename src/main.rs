use std::path::PathBuf;

use clap::Parser;
use image::{imageops::FilterType, DynamicImage, ImageBuffer, ImageFormat, Rgba};
use indicatif::ProgressBar;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// A poster/painting generation tool for Lethal Posters and Lethal Paintings
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The directory containing the poster and painting template images
    #[arg(short, long, default_value_t = String::from("./templates"))]
    templates: String,

    /// The directory containing the images to generate posters and paintings for
    #[arg(short, long, default_value_t = String::from("./input"))]
    input: String,

    /// The directory containing the images to generate posters and paintings for
    #[arg(short, long, default_value_t = String::from("./output"))]
    output: String,
}

const TEMPLATE_POSTER: &str = "poster_template.png";
const TEMPLATE_PAINTING: &str = "painting_template.png";
const POSTERS_OUT_DIR: &str = "BepInEx/plugins/LethalPosters/posters";
const TIPS_OUT_DIR: &str = "BepInEx/plugins/LethalPosters/tips";
const PAINTINGS_OUT_DIR: &str = "BepInEx/plugins/LethalPaintings/paintings";

fn main() {
    let args = Args::parse();
    println!("Parsed args: {args:?}");

    // Resolve paths
    println!("Resolving paths...");
    let template_dir = PathBuf::from(args.templates);
    let input_dir = PathBuf::from(args.input);
    let output_dir = PathBuf::from(args.output);
    let poster_template_path = get_path(&template_dir, TEMPLATE_POSTER);
    let painting_template_path = get_path(&template_dir, TEMPLATE_PAINTING);
    let posters_dir = create_dir_and_get_path(&output_dir, POSTERS_OUT_DIR);
    let tips_dir = create_dir_and_get_path(&output_dir, TIPS_OUT_DIR);
    let paintings_dir = create_dir_and_get_path(&output_dir, PAINTINGS_OUT_DIR);

    // Load images
    println!("Loading images...");
    let poster_template: DynamicImage = image::open(&poster_template_path)
        .unwrap_or_else(|e| panic!("Failed to open poster template image: {e:?}"));
    let painting_template: DynamicImage = image::open(&painting_template_path)
        .unwrap_or_else(|e| panic!("Failed to open painting template image: {e:?}"));
    let input_imgs = load_input_imgs(&input_dir);

    // Generate posters and paintings
    generate_assets(
        posters_dir,
        tips_dir,
        paintings_dir,
        poster_template,
        painting_template,
        input_imgs,
    );

    println!(
        "Operation complete! Images output to: {}",
        output_dir.to_str().unwrap_or("???")
    );
}

#[inline]
fn get_path(base: &PathBuf, sub_path: &str) -> PathBuf {
    let mut path = base.clone();
    path.push(sub_path);
    path
}

#[inline]
fn create_dir_and_get_path(base: &PathBuf, sub_path: &str) -> PathBuf {
    let path = get_path(base, sub_path);
    std::fs::create_dir_all(&path)
        .unwrap_or_else(|e| panic!("Failed to create output directory for \"{sub_path}\": {e:?}"));
    path
}

fn load_input_imgs(input_dir: &PathBuf) -> Vec<DynamicImage> {
    std::fs::read_dir(input_dir)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to read input directory \"{}\": {e:?}",
                input_dir.to_str().unwrap_or("???")
            )
        })
        .flat_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>()
        .par_iter()
        .filter_map(|path| image::open(path).ok())
        .collect()
}

fn generate_assets(
    posters_dir: PathBuf,
    tips_dir: PathBuf,
    paintings_dir: PathBuf,
    poster_template: DynamicImage,
    painting_template: DynamicImage,
    input_imgs: Vec<DynamicImage>,
) {
    // Create progress bar
    let img_count = input_imgs.len();
    println!("Generating {img_count} posters and paintings...");
    let bar: ProgressBar = ProgressBar::new(img_count as u64);

    // Generate posters/paintings
    input_imgs.par_iter().enumerate().for_each(|(i, _)| {
        // Output paths for this asset
        let img_name = format!("{i}.png");
        let poster_path = get_path(&posters_dir, &img_name);
        let tips_path = get_path(&tips_dir, &img_name);
        let painting_path = get_path(&paintings_dir, &img_name);

        rayon::scope(|s| {
            // Generate atlas
            s.spawn(|_| {
                generate_atlas(
                    &poster_template,
                    &[
                        &input_imgs[i % img_count],
                        &input_imgs[(i + 1) % img_count],
                        &input_imgs[(i + 2) % img_count],
                        &input_imgs[(i + 3) % img_count],
                        &input_imgs[(i + 4) % img_count],
                    ],
                )
                .save_with_format(&poster_path, ImageFormat::Png)
                .unwrap_or_else(|e| panic!("Failed to generate poster atlas: {e:?}"));
            });

            // Generate tips
            s.spawn(|_| {
                generate_tips(&input_imgs[i % img_count])
                    .save_with_format(&tips_path, ImageFormat::Png)
                    .unwrap_or_else(|e| panic!("Failed to generate tips: {e:?}"));
            });

            // Generate painting
            s.spawn(|_| {
                generate_painting(&painting_template, &input_imgs[i % img_count])
                    .save_with_format(&painting_path, ImageFormat::Png)
                    .unwrap_or_else(|e| panic!("Failed to generate painting: {e:?}"));
            });

            // Update progress bar
            bar.inc(1);
        })
    });

    // Finish progress bar
    bar.finish_with_message("Image generation complete!");
}

const POSTER_OFFSETS: &[&[u32; 4]; 5] = &[
    &[0, 0, 341, 559],
    &[346, 0, 284, 559],
    &[641, 58, 274, 243],
    &[184, 620, 411, 364],
    &[632, 320, 372, 672],
];

fn generate_atlas(template: &DynamicImage, posters: &[&DynamicImage; 5]) -> DynamicImage {
    let mut base = template.clone();

    // Generate overlays by resizing the image
    let overlays: Vec<(DynamicImage, (i64, i64))> = POSTER_OFFSETS
        .par_iter()
        .enumerate()
        .map(|(i, &o)| {
            let resized_img = posters[i].resize(o[2], o[3], FilterType::Lanczos3);
            let x = (o[0] + o[2] - resized_img.width()) as i64;
            let y = o[1] as i64;
            (resized_img, (x, y)) // Return the resized image and its overlay position
        })
        .collect();

    // Overlay image onto base
    // NOTE: Sequential because base cannot be mutated in parallel
    for (resized_poster, (x, y)) in overlays {
        image::imageops::overlay(&mut base, &resized_poster, x, y);
    }

    base
}

fn generate_tips(poster: &DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Create base image
    let mut base = ImageBuffer::new(796, 1024);

    // Resize poster image
    let resized_poster = poster.resize(796, 1024, FilterType::Lanczos3);

    // Overlay poster onto base
    let x = (796 - resized_poster.width()) as i64;
    image::imageops::overlay(&mut base, &resized_poster, x, 0);

    base
}

fn generate_painting(template: &DynamicImage, poster: &DynamicImage) -> DynamicImage {
    let mut base = template.clone();

    // Resize painting image
    let resized_painting = poster.resize_to_fill(243, 324, FilterType::Lanczos3);

    // Overlay painting onto base
    image::imageops::overlay(&mut base, &resized_painting, 264, 19);

    base
}
