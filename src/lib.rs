use std::error::Error;

use cairo::Context;
use regex::Regex;

#[derive(Debug)]
pub struct Command {
    cmd: char,
    args: Vec<(f64, f64)>,
}

impl Command {
    fn translate(&self, tx: f64, ty: f64) -> Self {
        let args = self
            .args
            .iter()
            .map(|(x, y)| (x + tx, y + ty))
            .collect::<Vec<_>>();
        Self {
            cmd: self.cmd,
            args,
        }
    }

    fn context(&self) -> (f64, f64) {
        *self.args.last().unwrap_or(&(0.0, 0.0))
    }

    fn to_str(&self, pt: (f64, f64)) -> (String, (f64, f64)) {
        match self.cmd {
            'M' => (
                format!(
                    "ctx.move_to({:0.6}, {:0.6});",
                    self.args[0].0, self.args[0].1
                ),
                (self.args[0].0, self.args[0].1),
            ),
            'C' => (
                format!(
                    "ctx.curve_to({:0.6}, {:0.6}, {:0.6}, {:0.6}, {:0.6}, {:0.6});",
                    self.args[0].0,
                    self.args[0].1,
                    self.args[1].0,
                    self.args[1].1,
                    self.args[2].0,
                    self.args[2].1
                ),
                (self.args[2].0, self.args[2].1),
            ),
            'L' => (
                format!(
                    "ctx.line_to({:0.6}, {:0.6});",
                    self.args[0].0, self.args[0].1
                ),
                (self.args[0].0, self.args[0].1),
            ),
            'H' => (
                format!("ctx.line_to({:0.6}, {:0.6});", self.args[0].0, pt.1),
                (self.args[0].0, pt.1),
            ),
            'V' => (
                format!("ctx.line_to({:0.6}, {:0.6});", pt.0, self.args[0].1),
                (pt.0, self.args[0].1),
            ),
            'z' | 'Z' => (String::from("ctx.close_path();"), (0.0, 0.0)),
            _ => (format!("// TODO: immplement {}", self.cmd), (0.0, 0.0)),
        }
    }

    fn from_str(s: &str, pt: (f64, f64)) -> Result<Self, Box<dyn Error>> {
        let cmd = s.chars().next().ok_or("empty command")?;
        let cmd = match cmd {
            'M' | 'm' | 'L' | 'l' | 'H' | 'h' | 'V' | 'v' | 'C' | 'c' | 'S' | 's' | 'Q' | 'q'
            | 'T' | 't' | 'A' | 'a' | 'Z' | 'z' => cmd,
            _ => return Err("invalid command".into()),
        };

        let s = s[1..].trim();
        if s.is_empty() {
            return Ok(Self {
                cmd,
                args: Vec::new(),
            });
        }

        let pattern = Regex::new(r"(\s|,)+")?;
        let args = pattern
            .split(s)
            .map(|s| s.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()?;
        let (px, py) = pt;
        match cmd {
            'H' => Ok(Self {
                cmd: 'L',
                args: vec![(args[0], py)],
            }),
            'V' => Ok(Self {
                cmd: 'L',
                args: vec![(px, args[0])],
            }),
            _ => Ok(Self {
                cmd,
                args: args.chunks(2).map(|x| (x[0], x[1])).collect::<Vec<_>>(),
            }),
        }
    }
}

impl std::str::FromStr for Command {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmd = s.chars().next().ok_or("empty command")?;
        let cmd = match cmd {
            'M' | 'm' | 'L' | 'l' | 'H' | 'h' | 'V' | 'v' | 'C' | 'c' | 'S' | 's' | 'Q' | 'q'
            | 'T' | 't' | 'A' | 'a' | 'Z' | 'z' => cmd,
            _ => return Err("invalid command".into()),
        };

        let s = s[1..].trim();
        if s.is_empty() {
            return Ok(Self {
                cmd,
                args: Vec::new(),
            });
        }

        let pattern = Regex::new(r"(\s|,)+")?;
        let args = pattern
            .split(s)
            .map(|s| s.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()?;
        match cmd {
            'H' => Ok(Self {
                cmd: 'H',
                args: vec![(args[0], 0.0)],
            }),
            'V' => Ok(Self {
                cmd: 'V',
                args: vec![(0.0, args[0])],
            }),
            _ => Ok(Self {
                cmd,
                args: args.chunks(2).map(|x| (x[0], x[1])).collect::<Vec<_>>(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Commands {
    cmds: Vec<Command>,
}

impl Commands {
    pub fn len(&self) -> usize {
        self.cmds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }

    pub fn bounds(&self) -> ((f64, f64), (f64, f64)) {
        let mut tl = (f64::MAX, f64::MAX);
        let mut br = (f64::MIN, f64::MIN);
        for cmd in &self.cmds {
            for (x, y) in &cmd.args {
                tl = (tl.0.min(*x), tl.1.min(*y));
                br = (br.0.max(*x), br.1.max(*y));
            }
        }
        (tl, br)
    }

    pub fn normalize(&self) -> Self {
        let ((tlx, tly), (brx, bry)) = self.bounds();
        let tx = -tlx - (brx - tlx) / 2.0;
        let ty = -tly - (bry - tly) / 2.0;

        Self {
            cmds: self
                .cmds
                .iter()
                .map(|cmd| cmd.translate(tx, ty))
                .collect::<Vec<_>>(),
        }
    }

    pub fn emit(&self) {
        let mut ctx = (0.0, 0.0);
        for cmd in &self.cmds {
            let (s, pt) = cmd.to_str((ctx.0, ctx.1));
            println!("{}", s);
            ctx = pt;
        }
    }
}

impl std::str::FromStr for Commands {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pt = (0.0, 0.0);
        let pattern = Regex::new(r"[MmLlHhVvCcSsQqTtAaZz][^MmLlHhVvCcSsQqTtAaZz]*")?;
        let mut cmds = Vec::new();
        for cmd in pattern.find_iter(s) {
            let cmd = Command::from_str(cmd.as_str(), pt)?;
            pt = cmd.context();
            cmds.push(cmd);
        }
        Ok(Self { cmds })
    }
}

pub trait Renderable {
    fn bounds() -> ((f64, f64), (f64, f64));
    fn create(ctx: &Context);
}

pub mod intuit_logo {
    use cairo::Context;

    pub fn bounds() -> ((f64, f64), (f64, f64)) {
        ((-726.73, -146.137), (726.73, 146.137))
    }

    pub fn create(ctx: &Context) {
        ctx.new_path();
        ctx.move_to(19.420000, 22.989000);
        ctx.curve_to(
            19.420000, 94.112000, 78.390000, 146.137000, 153.310000, 146.137000,
        );
        ctx.curve_to(
            228.220000, 146.137000, 287.260000, 94.112000, 287.260000, 22.989000,
        );
        ctx.line_to(287.260000, -140.413710);
        ctx.line_to(214.720000, -140.413710);
        ctx.line_to(214.720000, 14.565000);
        ctx.curve_to(
            214.720000, 52.956000, 188.100000, 78.808000, 153.110000, 78.808000,
        );
        ctx.curve_to(
            118.130000, 78.808000, 91.510000, 53.085000, 91.510000, 14.565000,
        );
        ctx.line_to(91.510000, -140.413710);
        ctx.line_to(18.970000, -140.413710);
        ctx.line_to(19.420000, 22.989000);
        ctx.close_path();
        ctx.move_to(481.080000, -75.013900);
        ctx.line_to(567.630000, -75.013900);
        ctx.line_to(567.630000, 140.735000);
        ctx.line_to(640.170000, 140.735000);
        ctx.line_to(640.170000, -75.013900);
        ctx.line_to(726.730000, -75.013900);
        ctx.line_to(726.730000, -140.542360);
        ctx.line_to(481.080000, -140.542360);
        ctx.line_to(481.080000, -75.013900);
        ctx.close_path();
        ctx.move_to(434.200000, -140.542360);
        ctx.line_to(361.660000, -140.542360);
        ctx.line_to(361.660000, 140.735000);
        ctx.line_to(434.200000, 140.735000);
        ctx.line_to(434.200000, -140.542360);
        ctx.close_path();
        ctx.move_to(-273.630000, -75.013900);
        ctx.line_to(-187.070000, -75.013900);
        ctx.line_to(-187.070000, 140.735000);
        ctx.line_to(-114.530000, 140.735000);
        ctx.line_to(-114.530000, -75.013900);
        ctx.line_to(-28.040000, -75.013900);
        ctx.line_to(-28.040000, -140.542360);
        ctx.line_to(-273.630000, -140.542360);
        ctx.line_to(-273.630000, -75.013900);
        ctx.close_path();
        ctx.move_to(-654.190000, -140.542360);
        ctx.line_to(-726.730000, -140.542360);
        ctx.line_to(-726.730000, 140.735000);
        ctx.line_to(-654.190000, 140.735000);
        ctx.line_to(-654.190000, -140.542360);
        ctx.close_path();
        ctx.move_to(-311.820000, -23.054000);
        ctx.curve_to(
            -311.820000,
            -94.177300,
            -370.790000,
            -146.137000,
            -445.770000,
            -146.137000,
        );
        ctx.curve_to(
            -520.760000,
            -146.137000,
            -579.730000,
            -94.177300,
            -579.730000,
            -23.054000,
        );
        ctx.line_to(-579.730000, 140.735000);
        ctx.line_to(-507.190000, 140.735000);
        ctx.line_to(-507.190000, -14.244000);
        ctx.curve_to(
            -507.190000,
            -52.635200,
            -480.560000,
            -78.486500,
            -445.580000,
            -78.486500,
        );
        ctx.curve_to(
            -410.600000,
            -78.486500,
            -383.980000,
            -52.763800,
            -383.980000,
            -14.244000,
        );
        ctx.line_to(-383.980000, 140.735000);
        ctx.line_to(-311.440000, 140.735000);
        ctx.line_to(-311.890000, -23.054000);
        ctx.line_to(-311.820000, -23.054000);
        ctx.close_path();
    }
}

pub mod empire_logo {
    use cairo::Context;

    pub fn bounds() -> ((f64, f64), (f64, f64)) {
        ((-300.0, -300.00004), (300.0, 300.00004))
    }

    pub fn create(ctx: &Context) {
        ctx.new_path();
        ctx.move_to(-0.000020, -300.000040);
        ctx.curve_to(
            -165.600000,
            -300.000040,
            -300.000000,
            -165.600000,
            -300.000000,
            0.000000,
        );
        ctx.curve_to(
            -300.000000,
            165.600000,
            -165.600020,
            300.000040,
            -0.000020,
            300.000000,
        );
        ctx.curve_to(
            165.599980, 300.000000, 300.000000, 165.600040, 300.000000, 0.000000,
        );
        ctx.curve_to(
            300.000000,
            -165.600020,
            165.599980,
            -300.000040,
            -0.000020,
            -300.000040,
        );
        ctx.close_path();
        ctx.move_to(-14.158180, -275.127560);
        ctx.curve_to(
            -13.989380,
            -275.136120,
            -13.816840,
            -275.119320,
            -13.647980,
            -275.127560,
        );
        ctx.line_to(-12.372460, -254.783180);
        ctx.curve_to(
            -29.675780,
            -253.958220,
            -46.517140,
            -251.434840,
            -62.755120,
            -247.321440,
        );
        ctx.line_to(-57.716840, -227.487240);
        ctx.curve_to(
            -100.338780,
            -216.691320,
            -138.302240,
            -194.245840,
            -168.048480,
            -163.711740,
        );
        ctx.line_to(-182.653080, -177.997440);
        ctx.curve_to(
            -194.531300,
            -165.811480,
            -205.150820,
            -152.425700,
            -214.413260,
            -138.073980,
        );
        ctx.line_to(-231.505100, -149.362260);
        ctx.curve_to(
            -184.824120,
            -221.539560,
            -105.309720,
            -270.502140,
            -14.158180,
            -275.127560,
        );
        ctx.close_path();
        ctx.move_to(13.647940, -275.127560);
        ctx.curve_to(
            105.011420,
            -270.658280,
            184.737680,
            -221.673240,
            231.505100,
            -149.362260,
        );
        ctx.line_to(214.413260, -138.073980);
        ctx.curve_to(
            205.171640,
            -152.389560,
            194.561880,
            -165.773220,
            182.716820,
            -177.933680,
        );
        ctx.line_to(168.112240, -163.711740);
        ctx.curve_to(
            138.361360,
            -194.262140,
            100.353640,
            -216.687580,
            57.716840,
            -227.487240,
        );
        ctx.line_to(62.755100, -247.321440);
        ctx.curve_to(
            46.517120,
            -251.434840,
            29.675780,
            -253.958220,
            12.372440,
            -254.783180,
        );
        ctx.line_to(13.647940, -275.127560);
        ctx.close_path();
        ctx.move_to(-0.000020, -185.204100);
        ctx.curve_to(
            8.673460,
            -185.223980,
            17.346960,
            -184.693900,
            24.489800,
            -183.673460,
        );
        ctx.line_to(13.329080, -90.816320);
        ctx.curve_to(
            36.788540, -87.395400, 57.356940, -75.117520, 71.556120, -57.461740,
        );
        ctx.line_to(146.045900, -113.329100);
        ctx.curve_to(
            155.094000,
            -102.057300,
            165.351620,
            -84.374340,
            170.727040,
            -70.982140,
        );
        ctx.line_to(85.140300, -34.375020);
        ctx.curve_to(
            89.442200, -23.746560, 91.836740, -12.165020, 91.836740, 0.000000,
        );
        ctx.curve_to(
            91.836740, 11.719600, 89.657980, 22.918700, 85.650500, 33.227040,
        );
        ctx.line_to(170.535700, 69.515320);
        ctx.curve_to(
            165.298120, 82.987060, 155.147500, 100.638460, 146.237240, 111.989800,
        );
        ctx.line_to(72.257640, 56.632660);
        ctx.curve_to(
            58.077340, 74.691760, 37.273400, 87.302640, 13.520380, 90.816320,
        );
        ctx.line_to(24.489800, 182.142860);
        ctx.curve_to(
            10.204080, 184.342840, -10.204080, 184.311260, -24.489800, 182.270420,
        );
        ctx.line_to(-13.520420, 90.816320);
        ctx.curve_to(
            -37.290580, 87.300100, -58.076680, 74.648900, -72.257660, 56.568880,
        );
        ctx.line_to(-146.045920, 111.926020);
        ctx.curve_to(
            -155.094000,
            100.654220,
            -165.351620,
            82.971260,
            -170.727060,
            69.579080,
        );
        ctx.line_to(-85.650520, 33.163260);
        ctx.curve_to(
            -89.644360, 22.870200, -91.836740, 11.698600, -91.836740, 0.000000,
        );
        ctx.curve_to(
            -91.836740, -12.186020, -89.456360, -23.795380, -85.140320, -34.438780,
        );
        ctx.line_to(-170.535720, -70.918380);
        ctx.curve_to(
            -165.298120,
            -84.390120,
            -155.147520,
            -102.041520,
            -146.237240,
            -113.392880,
        );
        ctx.line_to(-71.556140, -57.461740);
        ctx.curve_to(
            -57.356960, -75.117520, -36.788560, -87.395400, -13.329080, -90.816320,
        );
        ctx.line_to(-24.489800, -183.545940);
        ctx.curve_to(
            -17.346940,
            -184.645920,
            -8.673480,
            -185.184200,
            -0.000020,
            -185.204100,
        );
        ctx.close_path();
        ctx.move_to(-245.153060, -125.765300);
        ctx.line_to(-226.785720, -116.709180);
        ctx.curve_to(
            -234.532460,
            -101.690840,
            -240.866260,
            -85.834560,
            -245.535720,
            -69.260220,
        );
        ctx.line_to(-225.892860, -63.711740);
        ctx.curve_to(
            -231.603340,
            -43.446480,
            -234.693880,
            -22.083620,
            -234.693880,
            0.000000,
        );
        ctx.curve_to(
            -234.693880,
            22.105040,
            -231.614060,
            43.492780,
            -225.892860,
            63.775500,
        );
        ctx.line_to(-245.535720, 69.323980);
        ctx.curve_to(
            -240.869480,
            85.871640,
            -234.522940,
            101.713940,
            -226.785720,
            116.709180,
        );
        ctx.line_to(-245.153060, 125.765300);
        ctx.curve_to(
            -264.551240,
            88.053960,
            -275.510200,
            45.300820,
            -275.510200,
            0.000000,
        );
        ctx.curve_to(
            -275.510200,
            -45.300840,
            -264.551240,
            -88.053960,
            -245.153060,
            -125.765300,
        );
        ctx.close_path();
        ctx.move_to(245.153060, -125.765300);
        ctx.curve_to(
            264.551240, -88.053960, 275.510160, -45.300840, 275.510160, 0.000000,
        );
        ctx.curve_to(
            275.510160, 45.300820, 264.551240, 88.053960, 245.153060, 125.765300,
        );
        ctx.line_to(226.785700, 116.709180);
        ctx.curve_to(
            234.526200, 101.709480, 240.867940, 85.877040, 245.535720, 69.323980,
        );
        ctx.line_to(225.892860, 63.775500);
        ctx.curve_to(
            231.614040, 43.492780, 234.693880, 22.105040, 234.693880, 0.000000,
        );
        ctx.curve_to(
            234.693880, -22.083620, 231.603360, -43.446480, 225.892860, -63.711740,
        );
        ctx.line_to(245.535720, -69.260220);
        ctx.curve_to(
            240.867560,
            -85.829920,
            234.529260,
            -101.694440,
            226.785700,
            -116.709180,
        );
        ctx.line_to(245.153060, -125.765300);
        ctx.close_path();
        ctx.move_to(-214.413260, 138.074000);
        ctx.curve_to(
            -205.147280,
            152.425440,
            -194.531820,
            165.810920,
            -182.653080,
            177.997460,
        );
        ctx.line_to(-168.048480, 163.711740);
        ctx.curve_to(
            -138.302240,
            194.245820,
            -100.338780,
            216.691340,
            -57.716840,
            227.487260,
        );
        ctx.line_to(-62.755120, 247.321420);
        ctx.curve_to(
            -46.517140, 251.434800, -29.675780, 253.958220, -12.372460, 254.783180,
        );
        ctx.line_to(-13.647980, 275.127560);
        ctx.curve_to(
            -105.011420,
            270.658240,
            -184.737700,
            221.673200,
            -231.505100,
            149.362260,
        );
        ctx.line_to(-214.413260, 138.074000);
        ctx.close_path();
        ctx.move_to(214.477040, 138.074000);
        ctx.line_to(231.505100, 149.362260);
        ctx.curve_to(
            184.737680, 221.673200, 105.011420, 270.658240, 13.647940, 275.127560,
        );
        ctx.line_to(12.372440, 254.783180);
        ctx.curve_to(
            29.675780, 253.958220, 46.517120, 251.434800, 62.755100, 247.321420,
        );
        ctx.line_to(57.716840, 227.487260);
        ctx.curve_to(
            100.353640, 216.687560, 138.361360, 194.262120, 168.112240, 163.711740,
        );
        ctx.line_to(182.716820, 177.933680);
        ctx.curve_to(
            194.576700, 165.758000, 205.224120, 152.408920, 214.477040, 138.074000,
        );
    }
}
