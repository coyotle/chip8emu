mod chip8;

use bevy::prelude::*;
use chip8::Chip8;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the ROM file
    #[arg(short, long)]
    rom: PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("Loading ROM: {:?}", args.rom);

    let mut chip8 = Chip8::default();

    chip8.load_from_file(&args.rom);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(chip8)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_display)
        .add_systems(Update, (update_keys, draw_display))
        .add_systems(FixedUpdate, (run_chip8, update_chip8_timers))
        .add_systems(FixedUpdate, draw_registers)
        .run();
}

#[derive(Component)]
struct PcText;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    commands.spawn((
        Text::new("PC: 0x200\nOP: 0x0000"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        PcText,
    ));
}

const PIXEL_SIZE: f32 = 10.0;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

#[derive(Component)]
struct Chip8Pixel {
    x: usize,
    y: usize,
}

fn setup_display(mut commands: Commands) {
    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            commands.spawn((
                Sprite::from_color(
                    Color::srgb(0.0, 1.0, 0.0),
                    Vec2::new(PIXEL_SIZE, PIXEL_SIZE),
                ),
                Transform::from_xyz(
                    x as f32 * PIXEL_SIZE - DISPLAY_WIDTH as f32 * PIXEL_SIZE / 2.0,
                    DISPLAY_HEIGHT as f32 * PIXEL_SIZE / 2.0 - y as f32 * PIXEL_SIZE,
                    0.0,
                ),
                Chip8Pixel { x, y },
            ));
        }
    }
}

/// Обновляем состояние CHIP-8 с частотой 500Hz
fn run_chip8(mut chip8: ResMut<Chip8>, time: Res<Time>, mut accumulator: Local<f32>) {
    *accumulator += time.delta_secs();
    let cycle_time = 1.0 / 500.0;
    while *accumulator >= cycle_time {
        chip8.execute_opcode();
        *accumulator -= cycle_time;
    }
}

/// Обновляем таймеры CHIP-8 с частотой 60Hz
fn update_chip8_timers(mut chip8: ResMut<Chip8>, time: Res<Time>, mut accumulator: Local<f32>) {
    *accumulator += time.delta_secs();
    let timer_interval = 1.0 / 60.0;
    while *accumulator >= timer_interval {
        chip8.update_timers();
        *accumulator -= timer_interval;
    }
}

fn update_keys(keyboard_input: Res<ButtonInput<KeyCode>>, mut chip8: ResMut<Chip8>) {
    if keyboard_input.pressed(KeyCode::Escape) {
        chip8.restart();
    }

    let key_map = [
        KeyCode::KeyX,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::KeyQ,
        KeyCode::KeyW,
        KeyCode::KeyE,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::KeyZ,
        KeyCode::KeyC,
        KeyCode::Digit4,
        KeyCode::KeyR,
        KeyCode::KeyF,
        KeyCode::KeyV,
    ];

    for (i, &key_code) in key_map.iter().enumerate() {
        chip8.keys[i] = keyboard_input.pressed(key_code);
    }
}

// Система отрисовка дисплея
fn draw_display(chip8: Res<Chip8>, mut query: Query<(&Chip8Pixel, &mut Sprite)>) {
    for (px, mut sprite) in query.iter_mut() {
        if chip8.display[px.y][px.x] > 0 {
            sprite.color = Color::srgb(0.0, 1.0, 0.0);
        } else {
            sprite.color = Color::srgb(0.0, 0.0, 0.0);
        }
    }
}

fn draw_registers(chip8: Res<Chip8>, mut text: Single<&mut Text, With<PcText>>) {
    let pc = chip8.pc;
    let op = chip8.get_current_opcode();
    text.0 = format!("PC: {:04X}\nOP: {:04X}", pc, op);
}
