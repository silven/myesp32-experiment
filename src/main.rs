use embedded_hal::blocking::delay::DelayMs;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::i2c::config::MasterConfig;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::i2c::{self, I2c};
use esp_idf_hal::units::FromValueType;
use esp_idf_hal::gpio::{Pins, InputPin, OutputPin};
use esp_idf_hal::delay::FreeRtos;

use lcd_1602_i2c::{Lcd, Cursor, Blink};

use anyhow::{anyhow, Result};

// I suspect these shifts are to compensate for 7bit address mode or something?
const LCD_ADDRESS: u8 = 0x3E; // 0x7c >> 1;
const RGB_ADDRESS: u8 = 0x60; // 0xc0 >> 1;

fn main() {
    let peripherals = Peripherals::take().unwrap();
    let mut delay = FreeRtos {};

    display(
        peripherals.i2c0,
        peripherals.pins,
        &mut delay,
    ).unwrap();

    loop {
        delay.delay_ms(100u8);
    }
}

fn display(i2c0: i2c::I2C0, pins: Pins, delay: &mut impl DelayMs<u16>) -> Result<()> {
    let mut i2c_bus = i2c::Master::new(
        i2c0,
        i2c::MasterPins { 
            scl: pins.gpio10.into_output()?, 
            sda: pins.gpio1.into_input_output()?
        },
        MasterConfig::new()
            .baudrate(400.kHz().into()),
    )?;

    let addrs = i2c_scan(&mut i2c_bus);
    println!("Found i2c devices: {addrs:#x?}");

    let mut lcd = Lcd::new(i2c_bus, LCD_ADDRESS, RGB_ADDRESS, delay)?;
    lcd.set_rgb(255, 255, 255)?;
    lcd.set_cursor(Cursor::On)?;
    lcd.write_str("Hello, world!")?;
    lcd.set_blink(Blink::On)?;

    Ok(())
}

fn i2c_scan(i2c: &mut i2c::Master<impl I2c, impl InputPin + OutputPin, impl OutputPin>) -> Vec<u8>{
    use embedded_hal::prelude::_embedded_hal_blocking_i2c_Write;

    // I don't know if this command means anything, I just took them from the lcd1602 crate
    const LCD_4BITMODE: u8 = 0x00;
    const LCD_2LINE: u8 = 0x08;
    const LCD_5X8_DOTS: u8 = 0x00;
    const LCD_FUNCTIONSET: u8 = 0x20;
    const LCD_CMD: u8 = LCD_FUNCTIONSET | LCD_4BITMODE | LCD_2LINE | LCD_5X8_DOTS;

    let mut addrs = vec![];
    for addr in 0..=u8::MAX {
        if let Ok(_) = i2c.write(addr, &[0x80, LCD_CMD]) {
            addrs.push(addr);
        }
    }
    return addrs;
}
