use nrf52840_hal as hal;

use hal::{pac, Twim};

use lis3dh::{
    i2c::Lis3dh, Configuration as AccConfig, DataRate, InterruptSource, IrqPin1Conf, SlaveAddr,
};
pub use lis3dh::{Interrupt1, Lis3dhImpl};

type Twim0 = Twim<pac::TWIM0>;
type Lis3dhI2c = Lis3dh<Twim0>;

pub fn config_acc(
    twim: Twim0,
) -> Result<
    Lis3dhI2c,
    lis3dh::Error<<Lis3dhI2c as Lis3dhImpl>::BusError, <Lis3dhI2c as Lis3dhImpl>::PinError>,
> {
    let config = AccConfig {
        // Enable temperature readings. Device should run on 2V for temp sensor to work
        temp_en: false,
        // Continuously update data register
        block_data_update: true,
        datarate: DataRate::PowerDown,
        ..AccConfig::default()
    };

    // Initialize accelerometer using the config, passing spim2 and RTC-based delay
    let mut lis3dh: Lis3dhI2c = Lis3dh::new(twim, SlaveAddr::Default, config)?;

    // Configure the threshold value for interrupt 1 to 1.1g
    lis3dh.configure_irq_threshold(Interrupt1, 69)?;

    // The time in 1/ODR an axis value should be above threshold in order for an
    // interrupt to be raised
    lis3dh.configure_irq_duration(Interrupt1, 0)?;

    #[cfg(feature = "irq-ths")]
    {
        // Congfigure IRQ source for interrupt 1
        lis3dh.configure_irq_src(
            Interrupt1,
            InterruptSource {
                // use _or_ combnination, so interrupt is raised
                // if any one of the axes is above threshold
                and_or_combination: false,
                // Enable all axes, both high and low,
                // and activate the interrupt
                ..InterruptSource::all()
            },
            // latch irq line until src_register is read
            false,
            false,
        )?;

        // Configure IRQ pin 1
        lis3dh.configure_int_pin(IrqPin1Conf {
            // Raise if interrupt 1 is raised
            ia1_en: true,
            // Disable for all other interrupts
            ..IrqPin1Conf::default()
        })?;
    }
    #[cfg(feature = "irq-drdy")]
    {
        // Congfigure IRQ source for interrupt 1
        lis3dh.configure_irq_src(
            Interrupt1,
            InterruptSource::default(),
            // latch irq line until src_register is read
            false,
            false,
        )?;

        // Configure IRQ pin 1
        lis3dh.configure_int_pin(IrqPin1Conf {
            // Raise if interrupt 1 is raised
            ia1_en: true,
            // Rais interrupt 1 if accelerometer data is ready
            zyxda_en: true,
            // Disable for all other interrupts
            ..IrqPin1Conf::default()
        })?;
    }

    // Go to low power mode and wake up for 10*ODR if measurement above 1.1g is done
    lis3dh.configure_act(69, 10)?;

    lis3dh.set_datarate(DataRate::Hz_400)?;

    Ok(lis3dh)
}
