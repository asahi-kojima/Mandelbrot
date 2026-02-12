use num::Complex;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{PixelFormatEnum};
use std::io::{self, Write};
use std::time::Instant;


const MAX_FPS: f64 = 60.0;
const BAR_MAX_WIDTH: i32 = 20;
const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn main()
{


    let window_pixel_sizes = (WIDTH as usize, HEIGHT as usize);
    let mut upper_left = Complex { re: -1.20, im: 1.20 };
    let mut lower_right = Complex { re: 1.20, im: -1.20 };

    let mut pixels: Vec<u8> = vec![0; window_pixel_sizes.0 * window_pixel_sizes.1];

    let threads = 16;
    let rows_per_band = window_pixel_sizes.1 / threads + 1;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Mandelbrot", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH, HEIGHT)
        .unwrap();

    let mut last_time = Instant::now();
    let mut frame_count = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop 
    {
        for event in event_pump.poll_iter() 
        {
            match event
            {
                Event::Quit { .. } | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => 
                {
                    let height = lower_right.im - upper_left.im;
                    upper_left.im -= height * 0.1;
                    lower_right.im -= height * 0.1;
                }
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => 
                {
                    let height = lower_right.im - upper_left.im;
                    upper_left.im += height * 0.1;
                    lower_right.im += height * 0.1;
                }
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => 
                {
                    let width = lower_right.re - upper_left.re;
                    upper_left.re -= width * 0.1;
                    lower_right.re -= width * 0.1;
                }
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => 
                {
                    let width = lower_right.re - upper_left.re;
                    upper_left.re += width * 0.1;
                    lower_right.re += width * 0.1;
                }
                Event::KeyDown {keycode: Some(Keycode::A), ..} => 
                {
                    let width = lower_right.re - upper_left.re;
                    upper_left.re -= width * 0.1;
                    lower_right.re += width * 0.1;
                    let height = lower_right.im - upper_left.im;
                    upper_left.im -= height * 0.1;
                    lower_right.im += height * 0.1;
                }
                Event::KeyDown {keycode: Some(Keycode::S), ..} => 
                {
                    let width = lower_right.re - upper_left.re;
                    upper_left.re += width * 0.1;
                    lower_right.re -= width * 0.1;
                    let height = lower_right.im - upper_left.im;
                    upper_left.im += height * 0.1;
                    lower_right.im -= height * 0.1;
                }
                _ => {}
            }
        }


        // ----------------------------------------------
        // マンデルブロ集合の計算処理
        // ----------------------------------------------
        {
            let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * window_pixel_sizes.0).collect();
            crossbeam::scope(|spawner|
            {
                for (i, band) in bands.into_iter().enumerate()
                {
                    let top = rows_per_band * i;
                    let  height = band.len() / window_pixel_sizes.0;
                    let band_bounds = (window_pixel_sizes.0, height);
                    let band_upper_left = pixel_to_complex(window_pixel_sizes, (0,top), upper_left, lower_right);
                    let band_lower_right = pixel_to_complex(window_pixel_sizes, (window_pixel_sizes.0, top+height), upper_left, lower_right);

                    spawner.spawn(move |_| {
                        render(band, band_bounds, band_upper_left, band_lower_right);
                    });
                }
            }).unwrap();
        }

        // ----------------------------------------------
        // 書き込み処理
        // ----------------------------------------------
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize|{
            for y in 0..HEIGHT 
            {
                for x in 0..WIDTH 
                {
                    let offset: usize = y as usize * pitch + x as usize * 3;
                    let value = pixels[(y as usize) * window_pixel_sizes.0 + (x as usize)];
                    // ここで動的にピクセル値を計算（例：ランダムなノイズ）
                    let r = value;
                    let g = value;
                    let b = value;
                    
                    buffer[offset + 0] = r;
                    buffer[offset + 1] = g;
                    buffer[offset + 2] = b;
                }
            }
        }).unwrap();


        // ----------------------------------------------
        // 2. FPSの計測
        // ----------------------------------------------
        frame_count += 1;
        let elapsed_time = last_time.elapsed().as_secs_f64();
        if elapsed_time >= 1.0  
        {
            // println!("Diagnotis FPS: {}", frameCount);
            let ratio = (frame_count as f64 / elapsed_time / MAX_FPS).clamp(0.0, 1.0);
            let sub_block_num = (ratio * BAR_MAX_WIDTH as f64 * 8.0) as usize;
            let full_block_num = sub_block_num / 8;
            let rem_block_num = sub_block_num % 8;
            let mut fps_bar = String::with_capacity(BAR_MAX_WIDTH as usize);


            for _ in 0..full_block_num
            {
                fps_bar.push('\u{2588}'); // Full block
            }

            let partial_block = match rem_block_num {
                7 => '\u{2589}', // 7/8 block
                6 => '\u{258A}', // 6/8 block
                5 => '\u{258B}', // 5/8 block
                4 => '\u{258C}', // 4/8 block
                3 => '\u{258D}', // 3/8 block
                2 => '\u{258E}', // 2/8 block
                1 => '\u{258F}', // 1/8 block
                _ => ' ', // No block
            };
            fps_bar.push(partial_block);
            
            while fps_bar.chars().count() < BAR_MAX_WIDTH as usize 
            {
                fps_bar.push(' ');
            }

            print!("\x1b[2KFPS: {:3} \n\x1b[2K[{}]\x1b[1A\r", frame_count, fps_bar);

            io::stdout().flush().unwrap();

            frame_count = 0;
            last_time = Instant::now();
        }

        // ----------------------------------------------
        // 3. 描画
        // ----------------------------------------------
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
}

/// Noneの場合はマンデルブロ集合に含まれ、Some(i)が返された場合は発散したことを示す。
fn complex_square_add_loop(c : Complex<f64>, loop_limit : usize) -> Option<usize>
{
    let mut z: Complex<f64> = Complex { re: 0.0, im: 0.0 };
    for i in 0..loop_limit
    {
        if z.norm_sqr() > 4.0
        {
            return Some(i);
        }
        z = z * z + c;
    }
    
    None
}


fn pixel_to_complex(
    bounds : (usize, usize), 
    pixel : (usize, usize), 
    upper_left : Complex<f64>, 
    lower_right : Complex<f64>) -> Complex<f64>
{
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);
    if width <= 0.0 || height <= 0.0
    {
        panic!("Invalid complex plane region");
    }

    let re = upper_left.re + ((pixel.0 as f64) / (bounds.0 as f64)) * width ;
    let im = upper_left.im - ((pixel.1 as f64) / (bounds.1 as f64)) * height;
    Complex { re, im }
}

fn render(
    pixels : &mut [u8],
    bounds : (usize, usize), 
    upper_left : Complex<f64>, 
    lower_right : Complex<f64>)
{
    let mut max_brightness = 1u8;
    for row in 0..bounds.1
    {
        for col in 0..bounds.0
        {
            let point : Complex<f64> = pixel_to_complex(bounds, (col, row), upper_left, lower_right);
            let escape_time = complex_square_add_loop(point, 255);
            pixels[row * bounds.0 + col] = match escape_time
            {
                None => 0,
                Some(count) => 255 - count as u8,
            };
            max_brightness = max_brightness.max(pixels[row * bounds.0 + col]);
        }
    }
    for row in 0..bounds.1
    {
        for col in 0..bounds.0
        {
            let value = pixels[row * bounds.0 + col];
            pixels[row * bounds.0 + col] = (value as f32 / max_brightness as f32 * 255.0) as u8;
        }
    }
}
