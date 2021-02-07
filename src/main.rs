extern crate image;
extern crate piston_window;

#[macro_use]
extern crate clap;

use piston_window::*;

use clap::{App, Arg};
use image::GenericImageView;
use std::path::PathBuf;

struct CLIParams {
    first_image_path: PathBuf,
    second_image_path: PathBuf,
}

type Dimensions = (u32, u32);

pub fn get_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author("PicoJr")
        .about("Simple Image Diff Tool")
        .arg(
            Arg::with_name("first_image")
                .value_name("FIRST_IMAGE")
                .help("first image"),
        )
        .arg(
            Arg::with_name("second_image")
                .value_name("SECOND_IMAGE")
                .help("second image"),
        )
}

fn parse_args() -> anyhow::Result<CLIParams> {
    let matches = get_app().get_matches();
    let first_image = matches.value_of("first_image");
    let second_image = matches.value_of("second_image");
    match (first_image, second_image) {
        (Some(first_image_path), Some(second_image_path)) => Ok(CLIParams {
            first_image_path: PathBuf::from(first_image_path),
            second_image_path: PathBuf::from(second_image_path),
        }),
        _ => Err(anyhow::anyhow!("missing first_image or second_image")),
    }
}

fn check_dimensions(cli_params: &CLIParams) -> anyhow::Result<Dimensions> {
    let first_image = image::open(cli_params.first_image_path.as_path())?;
    let second_image = image::open(cli_params.second_image_path.as_path())?;

    if first_image.dimensions() != second_image.dimensions() {
        Err(anyhow::anyhow!(
            "first_image dimensions {:?} and second_image dimensions {:?} do not match",
            first_image.dimensions(),
            second_image.dimensions()
        ))
    } else {
        // same dimensions
        Ok(first_image.dimensions())
    }
}

fn build_textures(
    cli_params: &CLIParams,
    window: &mut PistonWindow,
) -> anyhow::Result<(G2dTexture, G2dTexture)> {
    let image_0_texture: G2dTexture = Texture::from_path(
        &mut window.create_texture_context(),
        &cli_params.first_image_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .map_err(|e| anyhow::anyhow!("{}", e))?;
    let image_1_texture: G2dTexture = Texture::from_path(
        &mut window.create_texture_context(),
        &cli_params.second_image_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok((image_0_texture, image_1_texture))
}

fn main() -> anyhow::Result<()> {
    let cli_params = parse_args()?;
    let dimensions = check_dimensions(&cli_params)?;

    let opengl = OpenGL::V4_5;
    let mut window: PistonWindow = WindowSettings::new("sdiff", [500, 500])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .vsync(true)
        .build()
        .unwrap();

    let (first_texture, second_texture) = build_textures(&cli_params, &mut window)?;

    let image = Image::new();

    let mut cut_at = dimensions.0 / 2;
    window.set_lazy(true);
    while let Some(e) = window.next() {
        if let Some(pos) = e.mouse_cursor_args() {
            cut_at = pos[0].abs() as u32;
        }

        let clamped_cut_at = if cut_at > dimensions.0 {
            dimensions.0
        } else {
            cut_at
        };
        window.draw_2d(&e, |c, g, _| {
            clear([1.0; 4], g);
            image
                .src_rect([0., 0., clamped_cut_at as f64, dimensions.1 as f64])
                .draw(&first_texture, &c.draw_state, c.transform, g);
            image
                .src_rect([
                    0. + clamped_cut_at as f64,
                    0.,
                    (dimensions.0 - clamped_cut_at) as f64,
                    dimensions.1 as f64,
                ])
                .draw(
                    &second_texture,
                    &c.draw_state,
                    c.transform.trans(0.0 + clamped_cut_at as f64, 0.0),
                    g,
                );
        });
    }
    Ok(())
}
