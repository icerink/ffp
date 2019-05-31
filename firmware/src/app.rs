// Copyright 2019 Adam Greig
// Dual licensed under the Apache 2.0 and MIT licenses.

use crate::hal;

#[derive(Copy, Clone)]
pub enum PinState {
    Low = 0,
    High = 1,
}

#[derive(Copy, Clone)]
pub enum Mode {
    HighImpedance = 0,
    Flash = 1,
    FPGA = 2,
}

#[derive(Copy, Clone)]
pub enum Request {
    SetCS(PinState),
    SetFPGA(PinState),
    SetTPwr(PinState),
    SetLED(PinState),
    SetMode(Mode),
    Transmit(([u8; 64], usize)),
    GetTPwr,
    Bootload,
    Suspend,
}

pub struct App<'a> {
    flash: &'a hal::flash::Flash,
    rcc: &'a hal::rcc::RCC,
    nvic: &'a hal::nvic::NVIC,
    dma: &'a hal::dma::DMA,
    pins: &'a hal::gpio::Pins<'a>,
    spi: &'a mut hal::spi::SPI,
    usb: &'a mut hal::usb::USB,
}

impl<'a> App<'a> {
    pub fn new(flash: &'a hal::flash::Flash, rcc: &'a hal::rcc::RCC,
               nvic: &'a hal::nvic::NVIC, dma: &'a hal::dma::DMA,
               pins: &'a hal::gpio::Pins<'a>, spi: &'a mut hal::spi::SPI,
               usb: &'a mut hal::usb::USB) -> Self {
        App {
            flash, rcc, nvic, dma, pins, spi, usb,
        }
    }

    pub fn setup(&mut self) {
        // Configure flash latency to 1 wait state with prefetch
        self.flash.setup();

        // Configure system clock to HSI48 and enable CRS and peripheral clocks
        self.rcc.setup();

        // Enable SEVONPEND
        self.nvic.setup();

        // Configure DMA for SPI1 transfers
        self.dma.setup();

        // Configure GPIOs
        self.pins.setup();

        // Configure SPI peripheral
        self.spi.setup();

        // Configure USB peripheral and connect to host
        self.usb.setup();
    }

    pub fn poll(&mut self) {
        /*
        // loop {
        self.pins.led.set_low();
        self.pins.led.set_high();
        // }
        */

        if self.nvic.usb_pending() {
            // Handle USB interrupts
            if let Some(req) = self.usb.interrupt() {
                self.process_request(req);
            }
            self.nvic.unpend_usb();
        } else {
            // Sleep until an interrupt occurs
            cortex_m::asm::wfe();
        }
    }

    fn process_request(&mut self, req: Request) {
        match req {
            Request::SetCS(state) => self.pins.cs.set_state(state),
            Request::SetFPGA(state) => self.pins.fpga_rst.set_state(state),
            Request::SetTPwr(state) => self.pins.tpwr_en.set_state(state),
            Request::SetLED(state) => self.pins.led.set_state(state),
            Request::SetMode(mode) => match mode {
                Mode::HighImpedance => {
                    self.pins.high_impedance_mode();
                    self.usb.disable_data_rx();
                },
                Mode::Flash => {
                    self.pins.flash_mode();
                    self.usb.enable_data_rx();
                },
                Mode::FPGA => {
                    self.pins.fpga_mode();
                    self.usb.enable_data_rx();
                },
            },
            Request::Transmit((data, n)) => {
                let rxdata = self.spi.exchange(&self.dma, &data[..n]);
                self.usb.reply_data(rxdata);
            },
            Request::GetTPwr => self.usb.reply_tpwr(self.pins.tpwr_det.get_state()),
            Request::Bootload => hal::bootload::bootload(),
            Request::Suspend => {
                self.pins.high_impedance_mode();
                self.pins.led.set_low();
                self.pins.tpwr_en.set_low();
            },
        };
    }
}
