use cairo::{Context, Format, ImageSurface};
use clap::Parser;
use intuit_empire::{logo_a, logo_b, Color};
use std::error::Error;
use std::fs;

#[derive(Debug, Clone)]
struct Size {
    width: i32,
    height: i32,
}

impl Size {
    fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    fn from_arg(s: &str) -> Result<Self, String> {
        s.parse::<Self>()
            .map_err(|_| format!("invalid size: {}", s))
    }
}

impl std::str::FromStr for Size {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ix = s.find('x').ok_or("invalid size")?;
        let width = s[..ix].parse()?;
        let height = s[ix + 1..].parse()?;
        Ok(Self { width, height })
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, value_parser = Size::from_arg, default_value_t = Size::new(3456,  2234))]
    size: Size,

    #[clap(long, default_value = "wallpaper.png")]
    dst: String,

    #[clap(long, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let surface = ImageSurface::create(Format::ARgb32, args.size.width, args.size.height)?;
    let ctx = Context::new(&surface)?;

    let w = args.size.width as f64;
    let h = args.size.height as f64;

    let cx = w / 2.0;
    let cy = h / 2.0;

    ctx.save()?;
    Color::from_rgb(35, 108, 255).set(&ctx);
    ctx.rectangle(0.0, 0.0, w, h);
    ctx.fill()?;
    ctx.restore()?;

    let is = 1.2;
    let es = 0.8;

    let ir = logo_a::bounds().scale(is, is);
    let er = logo_b::bounds().scale(es, es);
    let th = ir.height().max(er.height());
    let tw = ir.width() + er.width() + er.width() * 0.25;

    if args.debug {
        ctx.save()?;
        Color::from_u32(0xffffff).set(&ctx);
        ctx.translate(cx, cy);
        ctx.rectangle(-tw / 2.0, -th / 2.0, tw, th);
        ctx.stroke()?;
        ctx.restore()?;
    }

    ctx.save()?;
    Color::from_u32(0xffffff).set(&ctx);
    ctx.translate(cx - tw / 2.0 + er.width() / 2.0, cy);
    ctx.scale(es, es);
    logo_b::create(&ctx);
    ctx.fill()?;
    ctx.restore()?;

    ctx.save()?;
    Color::from_u32(0xffffff).set(&ctx);
    ctx.translate(cx + er.width() / 2.0 + er.width() * 0.125, cy);
    ctx.scale(is, is);
    logo_a::create(&ctx);
    ctx.fill()?;
    ctx.restore()?;

    if args.debug {
        ctx.save()?;
        Color::from_u32(0xffffff).set(&ctx);
        ctx.new_path();
        ctx.move_to(0.0, cy);
        ctx.line_to(args.size.width as f64, cy);
        ctx.move_to(cx, 0.0);
        ctx.line_to(cx, args.size.height as f64);
        ctx.stroke()?;
        ctx.restore()?;
    }

    surface.write_to_png(&mut fs::File::create(&args.dst)?)?;

    Ok(())
}
