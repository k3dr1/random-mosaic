use macroquad::{prelude::*, rand::gen_range};

fn pixel_distance(a: Color, b: Color) -> f32 {
    // sqrt(3) since these are vectors in R^3
    const NORMALIZATION_FACTOR: f32 = 1.7320508;
    ((a.r - b.r).powi(2) + (a.g - b.g).powi(2) + (a.b - b.b).powi(2)).sqrt() / NORMALIZATION_FACTOR
}

fn overlay_pixels(a: Color, b: Color) -> Color {
    Color {
        r: f32::min(a.r * (1. - b.a) + b.r * b.a, 1.),
        g: f32::min(a.g * (1. - b.a) + b.g * b.a, 1.),
        b: f32::min(a.b * (1. - b.a) + b.b * b.a, 1.),
        a: f32::min(a.a + b.a, 1.),
    }
}

fn region_distance(original_image: &Image, mutating_image: &Image, region: &Rect) -> f32 {
    assert!(original_image.width() == mutating_image.width());
    assert!(original_image.height() == mutating_image.height());
    assert!((region.x + region.w) <= original_image.width() as f32);
    assert!((region.y + region.h) <= original_image.height() as f32);

    let mut val = 0.0;
    for dy in 0..(region.h as u32) {
        for dx in 0..(region.w as u32) {
            let y = (region.y as u32) + dy;
            let x = (region.x as u32) + dx;
            val += pixel_distance(
                original_image.get_pixel(x, y),
                mutating_image.get_pixel(x, y),
            );
        }
    }
    val / (region.w * region.h)
}

fn calculate_mutation(
    original_image: &Image,
    mutating_image: &Image,
    mutating_shape: &Image,
    coord: (u32, u32),
) -> f32 {
    assert!(original_image.width() == mutating_image.width());
    assert!(original_image.height() == mutating_image.height());

    let mut val = 0.0;
    'outer: for dy in 0..(mutating_shape.height() as u32) {
        for dx in 0..(mutating_shape.width() as u32) {
            let gx = coord.0 + dx;
            let gy = coord.1 + dy;
            if gx >= original_image.width() as u32 || gy >= original_image.height() as u32 {
                break 'outer;
            }
            val += pixel_distance(
                original_image.get_pixel(gx, gy),
                overlay_pixels(
                    mutating_image.get_pixel(gx, gy),
                    mutating_shape.get_pixel(dx, dy),
                ),
            );
        }
    }
    val / (mutating_shape.width() * mutating_shape.height()) as f32
}

fn draw_image_to_image(to: &mut Image, from: &Image, region: &Rect) {
    assert!((region.x + region.w) <= to.width() as f32);
    assert!((region.y + region.h) <= to.height() as f32);

    for dy in 0..((region.h as u32) - 1) {
        for dx in 0..((region.w as u32) - 1) {
            let x = (region.x as u32) + dx;
            let y = (region.y as u32) + dy;
            let to_pixel = to.get_pixel(x, y);
            let from_pixel = from.get_pixel(dx, dy);
            let blended_pixel = overlay_pixels(to_pixel, from_pixel);
            to.set_pixel(x, y, blended_pixel);
        }
    }
}

fn create_shape(width: f32, height: f32) -> (Image, Rect) {
    let rel_w = 0.20;
    let rel_h = 0.20;
    let shape_rect = Rect {
        x: gen_range(0.0, width * (1.0 - rel_w)),
        y: gen_range(0.0, height * (1.0 - rel_h)),
        w: gen_range(0.0, width * rel_w),
        h: gen_range(0.0, height * rel_h),
    };
    let random_color = Color::new(
        gen_range(0.0, 1.0),
        gen_range(0.0, 1.0),
        gen_range(0.0, 1.0),
        gen_range(0.0, 1.0),
    );
    (
        Image::gen_image_color(shape_rect.w as u16, shape_rect.h as u16, random_color),
        shape_rect,
    )
}

#[macroquad::main("Mosaic")]
async fn main() {
    request_new_screen_size(600.0, 600.0);

    let original_image = load_image("images/fern4.png").await.unwrap();
    let mut mutating_image =
        Image::gen_image_color(original_image.width, original_image.height, BLACK);

    loop {
        let shapes: [(Image, Rect); 1024] = std::array::from_fn(|_| {
            create_shape(
                original_image.width() as f32,
                original_image.height() as f32,
            )
        });

        let mut drawn = false;
        for (shape, shape_rect) in shapes.into_iter() {
            let before = region_distance(&original_image, &mutating_image, &shape_rect);
            let after = calculate_mutation(
                &original_image,
                &mutating_image,
                &shape,
                (shape_rect.x as u32, shape_rect.y as u32),
            );
            if after < before {
                draw_image_to_image(&mut mutating_image, &shape, &shape_rect);
                drawn |= true;
            }
        }

        if !drawn {
            continue;
        }

        clear_background(DARKGRAY);

        draw_texture_ex(
            &Texture2D::from_image(&mutating_image),
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen_width() as f32, screen_height() as f32)),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        next_frame().await
    }
}
