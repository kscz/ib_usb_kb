#![no_std]
#![no_main]

use bsp::hal;
use itsybitsy_m4 as bsp;

use panic_halt as _;

use bsp::entry;
use hal::{
    clock::GenericClockController,
    delay::Delay,
    pac::{CorePeripherals, Peripherals},
    prelude::*,
    time::Hertz,
    timer::TimerCounter,
    timer_traits::InterruptDrivenTimer,
};
use smart_leds::{hsv::RGB8, SmartLedsWrite};
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_human_interface_device::page::Keyboard;
use usbd_human_interface_device::prelude::UsbHidClassBuilder;
use usbd_human_interface_device::prelude::*;


#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.gclk,
        &mut peripherals.mclk,
        &mut peripherals.osc32kctrl,
        &mut peripherals.oscctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = bsp::Pins::new(peripherals.port);
    let gclk0 = clocks.gclk0();
    let tc2_3 = clocks.tc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&tc2_3, peripherals.tc3, &mut peripherals.mclk);
    InterruptDrivenTimer::start(&mut timer, Hertz::MHz(4).into_duration());
    let mut rgb = bsp::dotstar_bitbang(
        pins.dotstar_miso.into(),
        pins.dotstar_mosi.into(),
        pins.dotstar_sck.into(),
        timer,
    );
    rgb.write([RGB8 { r: 0, g: 0, b: 120 }].iter().cloned())
        .unwrap();
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let usb_alloc = bsp::usb_allocator(
            peripherals.usb,
            &mut clocks,
            &mut peripherals.mclk,
            pins.usb_dm,
            pins.usb_dp,
        );

    let mut keyboard = UsbHidClassBuilder::new()
        .add_device(usbd_human_interface_device::device::keyboard::BootKeyboardConfig::default())
        .build(&usb_alloc);

    //https://pid.codes
    let mut usb_dev = UsbDeviceBuilder::new(&usb_alloc, UsbVidPid(0x1209, 0x0001))
        .strings(&[StringDescriptors::default()
            .manufacturer("usbd-human-interface-device")
            .product("Boot Keyboard")
            .serial_number("TEST")])
        .unwrap()
        .build();

    let mut type_cnt = 0u32;
    let test_type = [
        Keyboard::H,
        Keyboard::E,
        Keyboard::L,
        Keyboard::L,
        Keyboard::O,
        Keyboard::Space,
        Keyboard::W,
        Keyboard::O,
        Keyboard::R,
        Keyboard::L,
        Keyboard::D,
        ];

    let mut typeme = test_type.iter();

    loop {
        delay.delay_ms(1u8);
        match keyboard.tick() {
            Err(UsbHidError::WouldBlock) => {}
            Ok(_) => {}
            Err(e) => {
                core::panic!("Failed to process keyboard tick: {:?}", e)
            }
        };

        if type_cnt > 5000 && type_cnt % 60 == 50 {
            let keys = if let Some(x) = typeme.next() {
                [x.clone()]
            } else {
                [Keyboard::NoEventIndicated]
            };

            match keyboard.device().write_report(keys) {
                Err(UsbHidError::WouldBlock) => {}
                Err(UsbHidError::Duplicate) => {}
                Ok(_) => {}
                Err(e) => {
                    core::panic!("Failed to write keyboard report: {:?}", e)
                }
            };
        } else if type_cnt % 60 == 0 {
            let keys = [Keyboard::NoEventIndicated];

            match keyboard.device().write_report(keys) {
                Err(UsbHidError::WouldBlock) => {}
                Err(UsbHidError::Duplicate) => {}
                Ok(_) => {}
                Err(e) => {
                    core::panic!("Failed to write keyboard report: {:?}", e)
                }
            };
        }

        if usb_dev.poll(&mut [&mut keyboard]) {
            match keyboard.device().read_report() {
                Err(UsbError::WouldBlock) => {
                    //do nothing
                }
                Err(e) => {
                    core::panic!("Failed to read keyboard report: {:?}", e)
                }
                Ok(_leds) => {
                    // Numlock? Capslock?
                }
            }
        }

        type_cnt += 1;
    }
}

