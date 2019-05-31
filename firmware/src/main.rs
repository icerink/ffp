// Copyright 2019 Adam Greig
// Dual licensed under the Apache 2.0 and MIT licenses.

#![no_std]
#![no_main]

extern crate panic_halt;
use cortex_m_rt::{entry, pre_init};

pub mod hal;
pub mod app;

use app::App;

#[pre_init]
unsafe fn pre_init() {
    // Check if we should jump to system bootloader.
    //
    // When we receive the BOOTLOAD command over USB,
    // we write a flag to a static and reset the chip,
    // and `bootload::check()` will jump to the system
    // memory bootloader if the flag is present.
    //
    // It must be called from pre_init as otherwise the
    // flag is overwritten when statics are initialised.
    hal::bootload::check();
}

#[entry]
fn main() -> ! {
    // Obtain all required HAL instances
    let flash = hal::flash::Flash::new(stm32ral::flash::Flash::take().unwrap());
    let rcc = hal::rcc::RCC::new(stm32ral::rcc::RCC::take().unwrap());
    let nvic = hal::nvic::NVIC::new(stm32ral::nvic::NVIC::take().unwrap(),
                                    stm32ral::scb::SCB::take().unwrap());
    let dma = hal::dma::DMA::new(stm32ral::dma1::DMA1::take().unwrap());
    let gpioa = hal::gpio::GPIO::new(stm32ral::gpio::GPIOA::take().unwrap());
    let gpiob = hal::gpio::GPIO::new(stm32ral::gpio::GPIOB::take().unwrap());
    let mut spi = hal::spi::SPI::new(stm32ral::spi::SPI1::take().unwrap());
    let mut usb = hal::usb::USB::new(stm32ral::usb::USB::take().unwrap());

    // Define pinout
    let pins = hal::gpio::Pins {
        led: gpioa.pin(14),
        cs: gpiob.pin(1),
        fpga_rst: gpioa.pin(4),
        sck: gpioa.pin(5),
        flash_so: gpioa.pin(0),
        flash_si: gpioa.pin(1),
        fpga_so: gpiob.pin(6),
        fpga_si: gpiob.pin(7),

        tpwr_det: gpioa.pin(13),
        tpwr_en: gpioa.pin(3),
    };

    // Create App instance with the HAL instances
    let mut app = App::new(&flash, &rcc, &nvic, &dma, &pins, &mut spi, &mut usb);

    // Initialise application, including system peripherals
    app.setup();

    loop {
        // Process events
        app.poll();
    }
}
