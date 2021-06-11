#![no_std]
#![no_main]

use lis3dh::i2c::Lis3dh;
use lis3dh_irq_demo as _;
use lis3dh_irq_demo::acc;
use nrf52840_hal as hal;

use hal::{
    gpio::{p0::Parts, Floating, Input, Level, Output, Pin, PushPull},
    gpiote::Gpiote,
    pac::TWIM0,
    prelude::*,
    twim::Pins,
    Twim,
};

#[rtic::app(
    device=nrf52840_hal::pac,
    peripherals=true,
    monotonic=rtic::cyccnt::CYCCNT
)]
const APP: () = {
    struct Resources {
        gpiote: Gpiote,
        int1_pin: Pin<Input<Floating>>,
        led_1_pin: Pin<Output<PushPull>>,
        led_2_pin: Pin<Output<PushPull>>,
        led_3_pin: Pin<Output<PushPull>>,
        lis3dh: Lis3dh<Twim<TWIM0>>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> init::LateResources {
        let port0 = Parts::new(ctx.device.P0);
        let led_1_pin = port0.p0_13.into_push_pull_output(Level::High).degrade();
        let led_2_pin = port0.p0_14.into_push_pull_output(Level::High).degrade();
        let led_3_pin = port0.p0_15.into_push_pull_output(Level::High).degrade();
        let int1_pin = port0.p0_02.into_floating_input().degrade();

        let scl = port0.p0_27.into_floating_input().degrade();
        let sda = port0.p0_26.into_floating_input().degrade();
        let twim0 = Twim::new(
            ctx.device.TWIM0,
            Pins { scl, sda },
            hal::twim::Frequency::K400,
        );
        let lis3dh = acc::config_acc(twim0).unwrap();

        // Enable cycle counter for task scheduling
        ctx.core.DWT.enable_cycle_counter();

        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote
            .channel0()
            .input_pin(&int1_pin)
            .lo_to_hi()
            .enable_interrupt();
        gpiote
            .channel1()
            .input_pin(&int1_pin)
            .hi_to_lo()
            .enable_interrupt();

        init::LateResources {
            gpiote,
            int1_pin,
            led_1_pin,
            led_2_pin,
            led_3_pin,
            lis3dh,
        }
    }

    // Defines what happens when there's nothing left to do
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            // Go to sleep, waiting for an interrupt
            cortex_m::asm::wfi();
        }
    }

    /// Software task for setting LED 2 state
    #[task(resources = [led_2_pin], priority = 3)]
    fn set_led_2_state(ctx: set_led_2_state::Context, on: bool) {
        match on {
            true => ctx.resources.led_2_pin.set_low().unwrap(),
            false => ctx.resources.led_2_pin.set_high().unwrap(),
        }
    }

    #[task(resources = [lis3dh], priority = 3)]
    fn read_print_acc(ctx: read_print_acc::Context) {
        let lis3dh = ctx.resources.lis3dh;
        use lis3dh::accelerometer::Accelerometer;
        let sample = lis3dh.accel_norm().unwrap();
        defmt::info!(
            "Ouch! Sample: x: {}, y: {}, z: {}",
            sample.x,
            sample.y,
            sample.z
        );
    }

    /// Hardware task for handling GPIOTE events
    #[task(
        binds = GPIOTE,
        priority = 5,
        resources = [gpiote],
        spawn = [set_led_2_state, read_print_acc],
    )]
    fn on_gpiote(ctx: on_gpiote::Context) {
        let gpiote = ctx.resources.gpiote;

        if gpiote.channel0().is_event_triggered() {
            gpiote.channel0().reset_events();
            ctx.spawn.set_led_2_state(true).ok();
            ctx.spawn.read_print_acc().ok();
        }
        if gpiote.channel1().is_event_triggered() {
            gpiote.channel1().reset_events();
            ctx.spawn.set_led_2_state(false).ok();
        }
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these interrupts will be used to dispatch the
    // software tasks.
    // See https://rtic.rs/0.5/book/en/by-example/tasks.html;
    extern "C" {
        // Software interrupt 0 / Event generator unit 0
        fn SWI0_EGU0();
        // Software interrupt 1 / Event generator unit 1
        fn SWI1_EGU1();
        // Software interrupt 2 / Event generator unit 2
        fn SWI2_EGU2();
    }
};
