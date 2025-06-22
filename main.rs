// ! # GPIO 'Blinky' Example
//!
//! This application demonstrates how to control a GPIO pin on the RP2040.
//!
//! It may need to be adapted to your particular board layout and/or pin assignment.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

// Remove or guard any test-only code with #[cfg(test)] to avoid requiring the test crate in no_std binaries.

mod pong;
mod snake;

extern crate panic_halt;
extern crate embedded_hal;
extern crate rp2040_hal;
extern crate embedded_graphics;
extern crate embedded_time;
extern crate cortex_m_rt;
extern crate st7735_lcd;
extern crate fugit;
extern crate cortex_m;
extern crate heapless;
extern crate oorandom;

use cortex_m_rt::entry;
use heapless::Vec;
use core::fmt::Write;
use core::pin::Pin;
use cortex_m::interrupt::disable;
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Alias for our HAL crate
use rp2040_hal as hal;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use hal::pac;
use embedded_hal::adc::OneShot;

// Some traits we need
use embedded_hal::digital::v2::{InputPin, OutputPin};
use rp2040_hal::clocks::Clock;

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_time::fixed_point::FixedPoint;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::geometry::Size;
use embedded_time::rate::Extensions;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use rp2040_hal::{spi, Adc};
//use st7735_lcd;
use st7735_lcd::Orientation;
use embedded_graphics::prelude::*;
use embedded_graphics::draw_target::DrawTarget;
use fugit::RateExtU32;
use pong::{PlayerTurn, Pong, PLAYER_SIZE};
use snake::{Snake,Direction};

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
/// Note: This boot block is not necessary when using a rp-hal based BSP
/// as the BSPs already perform this step.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// The `#[rp2040_hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.

static mut ADC: Option<Adc> = None;
static mut MUX_SELECT_0: Option<hal::gpio::Pin<hal::gpio::bank0::Gpio10, hal::gpio::PushPullOutput>> = None;
static mut MUX_SELECT_1: Option<hal::gpio::Pin<hal::gpio::bank0::Gpio11, hal::gpio::PushPullOutput>> = None;
static mut MUX_JOY_ADC: Option<hal::gpio::Pin<hal::gpio::bank0::Gpio26, hal::gpio::FloatingInput>> = None;
const JOY_MAX_VAL: u16 = 4095;
const JOY_UPPER_BOUND: u16 = 3071; // 3/4 of JOY_MAX_VALUE
const JOY_LOWER_BOUND: u16 = 1024; // 1/4 of JOY_MAX_VALUE

#[rp2040_hal::entry]
unsafe fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    //adc pin for joysticks, since pico has only 3 adc we need to use muxer
    ADC = Some(hal::Adc::new(pac.ADC, &mut pac.RESETS));
    MUX_SELECT_0 = Some(pins.gpio10.into_push_pull_output());
    MUX_SELECT_1 = Some(pins.gpio11.into_push_pull_output());
    MUX_JOY_ADC = Some(pins.gpio26.into_floating_input());
    let mut joy_button1 = pins.gpio8.into_pull_up_input();
    let mut joy_button2 = pins.gpio9.into_pull_up_input();
    //lcd pins, spi communication, reset and light pins
    let _spi_sclk = pins.gpio6.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio7.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_miso = pins.gpio4.into_mode::<hal::gpio::FunctionSpi>();
    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);
    let mut lcd_led = pins.gpio12.into_push_pull_output();
    let dc = pins.gpio13.into_push_pull_output();
    let rst = pins.gpio14.into_push_pull_output();

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        RateExtU32::Hz(32_000_000u32),
        &embedded_hal::spi::MODE_0,
    );

    let mut disp = st7735_lcd::ST7735::new(spi, dc, rst, true, false, 160, 128);

    disp.init(&mut delay).unwrap();
    disp.set_orientation(&Orientation::Landscape).unwrap();
    disp.clear(Rgb565::BLACK).unwrap();

    let mut menu_change: bool = true;
    let mut score_changed: bool = true;
    let mut current_p1_score: u8 = 0;
    let mut current_p2_score: u8 = 0;
    let mut prev_player1: u16 = 128/2;
    let mut prev_player2: u16 = 128/2;
    let mut prev_ball: Point = Point::new(160/2, 128/2);
    let mut prev_snake: Vec<Point,100> = Vec::new();
    let mut prev_food: Vec<Point,100> = Vec::new();

    let mut current_state: CurrentState = CurrentState::Menu;
    loop {
        match current_state {
            CurrentState::Menu => {
                disp.clear(Rgb565::BLACK).unwrap();

                let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
                Text::new("Select Game", Point::new(40, 20), style)
                    .draw(&mut disp)
                    .unwrap();
                Text::new("> Pong", Point::new(40, 50), style)
                    .draw(&mut disp)
                    .unwrap();
                Text::new("  Snake", Point::new(40, 70), style)
                    .draw(&mut disp)
                    .unwrap();

                let mut selected_game = 0;
                loop {
                    let joy_val = read_joy(JoyToPin::JoyY1);
                    if joy_val > JOY_UPPER_BOUND {
                        menu_change = true;
                        selected_game = 1;
                    } else if joy_val < JOY_LOWER_BOUND {
                        menu_change = true;
                        selected_game = 0;
                    }
                    if menu_change {
                        disp.clear(Rgb565::BLACK).unwrap();
                        Text::new("Select Game", Point::new(40, 20), style)
                            .draw(&mut disp)
                            .unwrap();

                        if selected_game == 0 {
                            Text::new("> Pong", Point::new(40, 50), style)
                                .draw(&mut disp)
                                .unwrap();
                            Text::new("  Snake", Point::new(40, 70), style)
                                .draw(&mut disp)
                                .unwrap();
                        } else {
                            Text::new("  Pong", Point::new(40, 50), style)
                                .draw(&mut disp)
                                .unwrap();
                            Text::new("> Snake", Point::new(40, 70), style)
                                .draw(&mut disp)
                                .unwrap();
                        }
                        menu_change = false;
                    }
                    let confirm_val = joy_button1.is_low().unwrap();
                    if confirm_val {
                        if selected_game == 0 {
                            current_state = CurrentState::Pong(Pong::new(160, 128));
                        } else {
                            current_state = CurrentState::Snake(Snake::new(160,128));
                        }
                        disp.clear(Rgb565::BLACK).unwrap();
                        break;
                    }

                    delay.delay_ms(10);
                }
            }

            CurrentState::Pong(ref mut pong) => {
                let mut next_state: Option<CurrentState> = None;

                let paddle_style = PrimitiveStyle::with_fill(Rgb565::WHITE);
                let ball_style = PrimitiveStyle::with_fill(Rgb565::RED);
                let clear_style = PrimitiveStyle::with_fill(Rgb565::BLACK);

                //CLEANING PADDLES AND BALL OLD POSITIONS TO PREVENT FLICKERING
                Rectangle::new(
                    Point::new(0, prev_player1 as i32 - PLAYER_SIZE as i32),
                    Size::new(1, (PLAYER_SIZE * 2) as u32),
                )
                    .into_styled(clear_style)
                    .draw(&mut disp)
                    .unwrap();

                Rectangle::new(
                    Point::new(pong.width as i32 - 2, prev_player2 as i32 - PLAYER_SIZE as i32),
                    Size::new(1, (PLAYER_SIZE * 2) as u32),
                )
                    .into_styled(clear_style)
                    .draw(&mut disp)
                    .unwrap();

                // Clear old ball
                Rectangle::new(prev_ball, Size::new(2, 2))
                    .into_styled(clear_style)
                    .draw(&mut disp)
                    .unwrap();

                prev_player1 = pong.player1 as u16;
                prev_player2 = pong.player2 as u16;
                prev_ball = Point::new(pong.ball.x as i32, pong.ball.y as i32);

                Rectangle::new(
                    Point::new(0, pong.player1 as i32 - PLAYER_SIZE as i32),
                    Size::new(1, (PLAYER_SIZE * 2) as u32),
                )
                    .into_styled(paddle_style)
                    .draw(&mut disp)
                    .unwrap();

                Rectangle::new(
                    Point::new(pong.width as i32 - 2, pong.player2 as i32 - PLAYER_SIZE as i32),
                    Size::new(1, (PLAYER_SIZE * 2) as u32),
                )
                    .into_styled(paddle_style)
                    .draw(&mut disp)
                    .unwrap();

                Rectangle::new(
                    Point::new(pong.ball.x as i32, pong.ball.y as i32),
                    Size::new(2, 2),
                )
                    .into_styled(ball_style)
                    .draw(&mut disp)
                    .unwrap();

                if score_changed {
                    //CLEANING OLD SCORE WITH BLACK RECTANGLE
                    let clear_rect_style = PrimitiveStyle::with_fill(Rgb565::BLACK);
                    Rectangle::new(Point::new(70, 10), Size::new(15, 15))
                        .into_styled(clear_rect_style)
                        .draw(&mut disp)
                        .unwrap();
                    Rectangle::new(Point::new(90, 10), Size::new(15, 15))
                        .into_styled(clear_rect_style)
                        .draw(&mut disp)
                        .unwrap();

                    let mut buf1 = itoa::Buffer::new();
                    let mut buf2 = itoa::Buffer::new();
                    let p1_score = buf1.format(pong.player1_score);
                    let p2_score = buf2.format(pong.player2_score);

                    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
                    Text::new(p1_score, Point::new(70, 20), style)
                        .draw(&mut disp)
                        .unwrap();

                    Text::new(p2_score, Point::new(90, 20), style)
                        .draw(&mut disp)
                        .unwrap();

                    score_changed = false;
                }

                let p1_val = read_joy(JoyToPin::JoyY1);
                let p2_val = read_joy(JoyToPin::JoyY2);

                pong.move_player(PlayerTurn::Player1, p1_val as i16);
                pong.move_player(PlayerTurn::Player2, p2_val as i16);

                pong.update_ball();

                if current_p1_score != pong.player1_score || current_p2_score != pong.player2_score {
                    current_p1_score = pong.player1_score;
                    current_p2_score = pong.player2_score;

                    score_changed = true;
                }

                pong.check_for_win();

                if !pong.is_running {
                    next_state = Some(CurrentState::Menu);
                }

                if let Some(state) = next_state {
                    current_state = state;
                }
                delay.delay_ms(20);
            }

            CurrentState::Snake(ref mut snake) => {
                let mut next_state: Option<CurrentState> = None;

                let snake_style = PrimitiveStyle::with_fill(Rgb565::GREEN);
                let food_style = PrimitiveStyle::with_fill(Rgb565::RED);
                let clear_style = PrimitiveStyle::with_fill(Rgb565::BLACK);

                let prev_snake = snake.body.clone();
                let prev_food = snake.food.clone();

                for segment in prev_snake.iter() {
                    Rectangle::new(
                        Point::new(segment.x as i32, segment.y as i32),
                        Size::new(1,1)
                    )
                    .into_styled(clear_style)
                    .draw(&mut disp)
                    .unwrap();
                }

                for segment in prev_food.iter() {
                    Rectangle::new(
                        Point::new(segment.x as i32, segment.y as i32),
                        Size::new(1,1)
                    )
                    .into_styled(clear_style)
                    .draw(&mut disp)
                    .unwrap();
                }

                let player_xval = read_joy(JoyToPin::JoyY1);
                let player_yval = read_joy(JoyToPin::JoyX1);

                if player_xval > JOY_UPPER_BOUND {
                    snake.change_direction(Direction::Left);
                }
                else if player_xval < JOY_LOWER_BOUND {
                    snake.change_direction(Direction::Right);
                }
                else if player_yval > JOY_UPPER_BOUND {
                    snake.change_direction(Direction::Up);
                }
                else if player_yval < JOY_LOWER_BOUND {
                    snake.change_direction(Direction::Down);
                }

                snake.move_snake();
                snake.random_food_position();

                if snake.food.contains(&snake.head_position) {
                    snake.eat();
                    snake.random_food_position();
                    score_changed = true;
                }

                for segment in snake.body.iter() {
                    Rectangle::new(
                        Point::new(segment.x as i32, segment.y as i32),
                        Size::new(1,1)
                    )
                        .into_styled(snake_style)
                        .draw(&mut disp)
                        .unwrap();
                }

                for segment in snake.food.iter() {
                    Rectangle::new(
                        Point::new(segment.x as i32, segment.y as i32),
                        Size::new(1,1)
                    )
                        .into_styled(food_style)
                        .draw(&mut disp)
                        .unwrap();
                }

                if score_changed {
                    //CLEANING OLD SCORE WITH BLACK RECTANGLE
                    let clear_rect_style = PrimitiveStyle::with_fill(Rgb565::BLACK);
                    Rectangle::new(Point::new(70, 10), Size::new(15, 15))
                        .into_styled(clear_rect_style)
                        .draw(&mut disp)
                        .unwrap();
                    Rectangle::new(Point::new(90, 10), Size::new(15, 15))
                        .into_styled(clear_rect_style)
                        .draw(&mut disp)
                        .unwrap();

                    let mut buf1 = itoa::Buffer::new();
                    let p1_score = buf1.format(snake.score);

                    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
                    Text::new(p1_score, Point::new(70, 20), style)
                        .draw(&mut disp)
                        .unwrap();

                    score_changed = false;
                }

                if !snake.alive {
                    next_state = Some(CurrentState::Menu)
                }

                if let Some(state) = next_state {
                    current_state = state;
                }
                delay.delay_ms(50);
            }
        }
    }
}

pub enum CurrentState {
    Menu,
    Pong(Pong),
    Snake(Snake),
}

pub enum JoyToPin {
    JoyX1 = 0,
    JoyY1 = 1,
    JoyX2 = 2,
    JoyY2 = 3
}

fn read_joy(joy: JoyToPin) -> u16 {
    let (s0, s1) = match joy {
        JoyToPin::JoyX1 => (false, false),
        JoyToPin::JoyY1 => (true, false),
        JoyToPin::JoyX2 => (false, true),
        JoyToPin::JoyY2 => (true, true),
    };

    unsafe {
        let adc = ADC.as_mut().unwrap();
        let mux_joy_adc = MUX_JOY_ADC.as_mut().unwrap();
        let mux_select_0 = MUX_SELECT_0.as_mut().unwrap();
        let mux_select_1 = MUX_SELECT_1.as_mut().unwrap();

        if s0 {
            mux_select_0.set_high().unwrap();
        } else {
            mux_select_0.set_low().unwrap();
        }

        if s1 {
            mux_select_1.set_high().unwrap();
        } else {
            mux_select_1.set_low().unwrap();
        }

        cortex_m::asm::delay(10000);

        adc.read(mux_joy_adc).unwrap_or(0)
    }
}