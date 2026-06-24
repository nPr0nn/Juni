// Run natively:   cargo run   (or: cargo run --example shapes)
// Run on the web: trunk serve --open                  (WebGPU)
//                 trunk serve --features webgl --open (WebGL fallback)

use juni::prelude::*;

// A custom fragment shader: an animated rainbow driven by world position and
// `globals.time`. Same vertex/uniform interface as the built-in shape shader,
// so it plugs straight into `begin_shader_mode`. Kept inline (not `include_str!`)
// because this file is also `include!`d by src/main.rs, which would break a
// relative shader path.
const RAINBOW_SHADER: &str = r#"
struct Globals {
    proj: mat4x4<f32>,
    time: f32,
};
@group(0) @binding(0) var<uniform> globals: Globals;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VsOut {
    var out: VsOut;
    out.clip = globals.proj * vec4<f32>(position, 0.0, 1.0);
    out.world = position;
    return out;
}

// Hue (0..1) -> RGB.
fn hue(h: f32) -> vec3<f32> {
    let r = abs(h * 6.0 - 3.0) - 1.0;
    let g = 2.0 - abs(h * 6.0 - 2.0);
    let b = 2.0 - abs(h * 6.0 - 4.0);
    return clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let h = fract((in.world.x + in.world.y) * 0.0015 + globals.time * 0.2);
    return vec4<f32>(hue(h), 1.0);
}
"#;

struct Demo {
    x: f32,
    dir: f32,
    player: Vec2D,
    mouse: Vec2D,
    rainbow: Shader,
}

impl Game for Demo {
    fn init(ctx: &mut Context) -> Self {
        Demo {
            x: 100.0, dir: 1.0,
            player: Vec2D{ x: 0.0, y: 0.0 },
            mouse: Vec2D::ZERO,
            // Compile the custom shader once, up front (raylib's LoadShader).
            rainbow: ctx.load_shader_from_memory(RAINBOW_SHADER),
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        // Track the cursor in virtual-canvas coordinates.
        self.mouse = ctx.mouse_position();

        println!("{}", ctx.dt);

        // Press F to toggle fullscreen, Esc to quit.
        if ctx.is_key_pressed(Key::F) {
            ctx.toggle_fullscreen();
        }
        if ctx.is_key_pressed(Key::Escape) {
            ctx.exit();
        }

        // Player Movement
        if ctx.is_key_down(Key::W) {
            self.player.y -= 5.0;
        }
        if ctx.is_key_down(Key::A) {
            self.player.x -= 5.0;
        }
        if ctx.is_key_down(Key::S) {
            self.player.y += 5.0;
        }
        if ctx.is_key_down(Key::D) {
            self.player.x += 5.0;
        }

        // Fixed-timestep movement: 240 virtual px/sec, bouncing in [100, 1080].
        self.x += self.dir * 240.0 * ctx.dt;
        if self.x > 1080.0 {
            self.x = 1080.0;
            self.dir = -1.0;
        } else if self.x < 100.0 {
            self.x = 100.0;
            self.dir = 1.0;
        }
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        canvas.clear_background(WHITE);

        // Static rectangle.
        canvas.rectangle(60.0, 60.0, 300.0, 180.0, SKYBLUE);

        // Rectangle from a Rectangle struct.
        canvas.rectangle_from_rect(Rect::new(60.0, 300.0, 300.0, 180.0), GOLD);

        // A triangle.
        canvas.triangle(
            Vec2D::new(640.0, 120.0),
            Vec2D::new(540.0, 320.0),
            Vec2D::new(740.0, 320.0),
            MAROON,
        );

        // A free-form quad (parallelogram).
        canvas.quad(
            Vec2D::new(820.0, 120.0),
            Vec2D::new(1120.0, 120.0),
            Vec2D::new(1060.0, 320.0),
            Vec2D::new(760.0, 320.0),
            DARKGREEN,
        );

        // An animated rainbow quad, drawn with a custom shader. Everything
        // between begin/end_shader_mode uses `self.rainbow` instead of the
        // default shape shader (raylib's BeginShaderMode / EndShaderMode). The
        // quad's own vertex color is ignored — the fragment shader computes it.
        canvas.begin_shader_mode(&self.rainbow);
        canvas.quad(
            Vec2D::new(400.0, 300.0),
            Vec2D::new(500.0, 300.0),
            Vec2D::new(500.0, 480.0),
            Vec2D::new(400.0, 480.0),
            RED,
        );
        canvas.end_shader_mode();

        // A circle and a regular polygon (pentagon).
        canvas.circle(Vec2D::new(960.0, 480.0), 70.0, PURPLE);
        canvas.regular_polygon(Vec2D::new(1140.0, 480.0), 5, 70.0, -90.0, ORANGE);

        // A thick line from the canvas center to the mouse cursor.
        canvas.line(Vec2D::new(640.0, 360.0), self.mouse, 5.0, DARKBLUE);

        // The moving rectangle.
        canvas.rectangle(self.x, 520.0, 100.0, 100.0, RED);
        canvas.rectangle(self.player.x, self.player.y, 100.0, 100.0, BLACK);
    }
}

fn main() {
    run::<Demo>(Config {
        width: 1080,
        height: 720,
        render_width: 1080,
        render_height: 720,
        title: "juni — shapes".to_string(),
        target_ups: 60,
        centered: true,
        resizable: false,
        // 4x MSAA looks crisp on native but is expensive on the web (WebGL2
        // resolves are bandwidth-heavy), so disable it there.
        msaa: if cfg!(target_arch = "wasm32") { 1 } else { 4 },
        ..Config::default()
    });
}
