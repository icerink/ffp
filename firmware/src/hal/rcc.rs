// Copyright 2019 Adam Greig
// Dual licensed under the Apache 2.0 and MIT licenses.

use stm32ral::rcc;
use stm32ral::{read_reg, modify_reg};
use cortex_m;

pub struct RCC {
    rcc: rcc::Instance,
}

impl RCC {
    pub fn new(rcc: rcc::Instance) -> Self {
        RCC { rcc }
    }

    /// Set up the device, enabling all required clocks
    pub fn setup(&self) {
        // clean clock bypass
        modify_reg!(rcc, self.rcc, CR, HSEBYP: NotBypassed);

        // clear all the interrupt enaobles
        modify_reg!(rcc, self.rcc, CIR, LSIRDYIE: Disabled);
        modify_reg!(rcc, self.rcc, CIR, LSERDYIE: Disabled);
        modify_reg!(rcc, self.rcc, CIR, HSIRDYIE: Disabled);
        modify_reg!(rcc, self.rcc, CIR, HSERDYIE: Disabled);
        modify_reg!(rcc, self.rcc, CIR, PLLRDYIE: Disabled);
        modify_reg!(rcc, self.rcc, CIR, HSI14RDYIE: Disabled);
        modify_reg!(rcc, self.rcc, CIR, HSI48RDYIE: Disabled);

        // Turn on HSE (8 MHz)
        modify_reg!(rcc, self.rcc, CR, HSEON: On);

        // Turn on HSI
        modify_reg!(rcc, self.rcc, CR, HSION: On);

        // Turn on clock security system
        modify_reg!(rcc, self.rcc, CR, CSSON: On);

        // Wait for HSE to be ready
        while read_reg!(rcc, self.rcc, CR, HSERDY == NotReady) {}

        // Wait for HSI to be ready
        while read_reg!(rcc, self.rcc, CR, HSIRDY == NotReady) {}


        // swap to internal oscillator if PLL is currently used
        if read_reg!(rcc, self.rcc, CFGR, SWS == PLL) {
            modify_reg!(rcc, self.rcc, CFGR, SW: HSI);

            while read_reg!(rcc, self.rcc, CFGR, SWS != HSI) {}
        }

        // turn off pll
        modify_reg!(rcc, self.rcc, CR, PLLON: Off);
        // wait for the pll to actually turn off
        while read_reg!(rcc, self.rcc, CR, PLLRDY == Ready) {}

        // Select HSE as PLL input
        modify_reg!(rcc, self.rcc, CFGR, PLLSRC: HSE_Div_PREDIV);
        // pll multiplication
        modify_reg!(rcc, self.rcc, CFGR, PLLMUL: Mul6);
        // pll division
        modify_reg!(rcc, self.rcc, CFGR2, PREDIV: Div1);

        // Turn on PLL
        modify_reg!(rcc, self.rcc, CR, PLLON: On);

        // Wait for PLL to be ready
        while read_reg!(rcc, self.rcc, CR, PLLRDY == NotReady) {}

        // Swap system clock to PLL
        modify_reg!(rcc, self.rcc, CFGR, SW: PLL);
        // Wait for system clock to be PLL
        while read_reg!(rcc, self.rcc, CFGR, SWS != PLL) {}

        // pclk divider
        modify_reg!(rcc, self.rcc, CFGR, PPRE: Div1);
        // hclk divider
        modify_reg!(rcc, self.rcc, CFGR, HPRE: Div1);

        // Enable peripheral clocks
        modify_reg!(rcc, self.rcc, AHBENR, IOPAEN: Enabled, IOPBEN: Enabled, DMAEN: Enabled);
        modify_reg!(rcc, self.rcc, APB2ENR, SPI1EN: Enabled);

        // select pll for usb
        modify_reg!(rcc, self.rcc, CFGR3, USBSW: PLLCLK);
        
        // enable usb 
        modify_reg!(rcc, self.rcc, APB1ENR, USBEN: Enabled);

        // reset usb
        modify_reg!(rcc, self.rcc, APB1RSTR, USBRST: Reset);

        let mut i = 0;

        while i < 4000 {
            i += 1;
            // Wait t_STARTUP (1µs)
            cortex_m::asm::delay(48);
        }

        // unreset usb
        modify_reg!(rcc, self.rcc, APB1RSTR, USBRST: 0);
    }
}
