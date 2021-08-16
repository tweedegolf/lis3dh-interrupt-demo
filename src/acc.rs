use lis3dh::{Configuration as AccConfig, DataRate, IrqPin1Conf, Lis3dh, SlaveAddr};
pub use lis3dh::{Interrupt1, Lis3dhI2C};

use embedded_hal::blocking::i2c::{self, WriteRead};

pub fn config_acc<Twim, E>(
    twim: Twim,
) -> Result<Lis3dh<Lis3dhI2C<Twim>>, lis3dh::Error<E, core::convert::Infallible>>
where
    Twim: WriteRead<Error = E> + i2c::Write<Error = E>,
{
    let config = AccConfig {
        datarate: DataRate::PowerDown,
        ..AccConfig::default()
    };

    // Initialize accelerometer using the config
    let mut lis3dh = Lis3dh::new_i2c_with_config(twim, SlaveAddr::Default, config)?;

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
            lis3dh::InterruptMode::Movement,
            lis3dh::InterruptConfig::high_and_low(),
        )?;

        // Configure IRQ pin 1
        lis3dh.configure_interrupt_pin(IrqPin1Conf {
            // Raise if interrupt 1 is raised
            ia1_en: true,
            // Disable for all other interrupts
            ..IrqPin1Conf::default()
        })?;
        // Go to low power mode and wake up for 10*ODR if measurement above 1.1g is done
        lis3dh.configure_switch_to_low_power(69, 10)?;

        lis3dh.set_datarate(DataRate::Hz_400)?;
    }

    #[cfg(feature = "irq-drdy")]
    {
        // Congfigure IRQ source for interrupt 1
        lis3dh.configure_irq_src(
            Interrupt1,
            lis3dh::InterruptMode::Movement,
            lis3dh::InterruptConfig::high_and_low(),
        )?;

        // Configure IRQ pin 1
        lis3dh.configure_interrupt_pin(IrqPin1Conf {
            // Raise if interrupt 1 is raised
            ia1_en: true,
            // Rais interrupt 1 if accelerometer data is ready
            zyxda_en: true,
            // Disable for all other interrupts
            ..IrqPin1Conf::default()
        })?;
        lis3dh.set_datarate(DataRate::Hz_1)?;
    }

    Ok(lis3dh)
}
