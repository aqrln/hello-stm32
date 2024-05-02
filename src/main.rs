#![no_std]
#![no_main]

mod fmt;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::{Duration, Timer};
use fmt::*;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut led_blue = Output::new(p.PC13, Level::High, Speed::Low);
    let mut led_red = Output::new(p.PB8, Level::Low, Speed::Low);
    let mut led_warning = Output::new(p.PB10, Level::Low, Speed::Low);

    let btn = Input::new(p.PA0, Pull::Up);

    const SEQ_TIME_MS: usize = 5000;
    const TICK_TIME_MS: usize = 100;
    const TICKS: usize = SEQ_TIME_MS / TICK_TIME_MS;
    const MIN_IGNORED_TICKS_AFTER_RECORDING: usize = 5;

    let mut levels = [Level::High; TICKS];
    let mut tick = 0;

    let wait = || Timer::after(Duration::from_millis(TICK_TIME_MS as u64));

    loop {
        info!("btn level: {}", btn.get_level());

        led_blue.set_level(levels[tick]);
        led_red.set_level(levels[tick]);

        tick += 1;
        if tick >= levels.len() {
            tick = 0;
        }

        if btn.is_high() {
            wait().await;
            continue;
        }

        for (i, saved_level) in levels.iter_mut().enumerate() {
            info!("recording {}/{}", i, TICKS);
            let btn_level = btn.get_level();
            *saved_level = btn_level;

            led_blue.set_level(btn_level);
            led_red.set_level(btn_level);

            wait().await;
        }

        led_warning.set_high();

        for _ in 0..MIN_IGNORED_TICKS_AFTER_RECORDING {
            wait().await;
        }

        // prevent accidental restart of recording: if button is still pressed, wait until it's released
        while btn.is_low() {
            led_warning.toggle();
            wait().await;
        }

        led_warning.set_low();

        tick = 0;
        wait().await;
    }
}
