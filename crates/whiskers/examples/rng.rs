use rand::Rng;
use whiskers::prelude::*;

#[derive(Sketch)]
struct RngSketch {
    width: f64,
    height: f64,
}

impl Default for RngSketch {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 300.0,
        }
    }
}

impl App for RngSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.color(Color::DARK_RED).stroke_width(3.0);

        let should_generate_random_color = ctx.rng_boolean();

        println!(
            "Was a random color generated? {}",
            if should_generate_random_color {
                "Yes"
            } else {
                "No"
            }
        );

        let chosen_color;

        let colors: Vec<Color> = vec![
            Color::BLACK,
            Color::DARK_GRAY,
            Color::GRAY,
            Color::LIGHT_GRAY,
            Color::WHITE,
            Color::BROWN,
            Color::DARK_RED,
            Color::RED,
            Color::LIGHT_RED,
            Color::YELLOW,
            Color::LIGHT_YELLOW,
            Color::KHAKI,
            Color::DARK_GREEN,
            Color::GREEN,
            Color::LIGHT_GREEN,
            Color::DARK_BLUE,
            Color::BLUE,
            Color::LIGHT_BLUE,
            Color::GOLD,
        ];
        chosen_color = if should_generate_random_color {
            ctx.rng_option(&colors).unwrap()
        } else {
            &Color::BLACK
        };

        println!("{}", chosen_color.to_rgb_string());

        sketch.color(Color::from(*chosen_color));
        sketch
            .translate(sketch.width() / 2.0, sketch.height() / 2.0)
            .rect(0., 0., self.width, self.height);
        Ok(())
    }
}

fn main() -> Result {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen();
    Runner::new(RngSketch::default())
        .with_seed(num)
        .with_page_size_options(PageSize::A5H)
        .run()
}
