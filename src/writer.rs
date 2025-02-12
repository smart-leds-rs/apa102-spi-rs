use crate::{Apa102Pixel, PixelOrder};

use super::{bisync, SmartLedsWrite, SpiBus};

/// A writer for APA102 LEDs
#[bisync]
pub struct Apa102<SPI> {
    spi: SPI,
    invert_end_frame: bool,
    pixel_order: PixelOrder,
}

#[bisync]
impl<SPI> Apa102<SPI>
where
    SPI: SpiBus,
{
    /// Construct a writer for APA102 LEDs.
    /// If your LEDs don't use the standard BGR ordering, use [Self::new_with_options] instead.
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            invert_end_frame: true,
            pixel_order: PixelOrder::BGR,
        }
    }

    pub fn new_with_options(
        spi: SPI,
        invert_end_frame: bool,
        pixel_order: PixelOrder,
    ) -> Self {
        Self {
            spi,
            invert_end_frame,
            pixel_order,
        }
    }

    /// Free the owned resources consuming self
    pub fn free(self) -> SPI {
        self.spi
    }
}

#[bisync]
impl<SPI> SmartLedsWrite for Apa102<SPI>
where
    SPI: SpiBus,
{
    type Color = Apa102Pixel;
    type Error = SPI::Error;
    /// Write all the items of an iterator to an apa102 strip
    async fn write<T, I>(&mut self, iterator: T) -> Result<(), SPI::Error>
    where
        T: IntoIterator<Item = I, IntoIter: ExactSizeIterator>,
        I: Into<Self::Color>,
    {
        self.spi.write(&[0x00, 0x00, 0x00, 0x00]).await?;
        let iterator = iterator.into_iter();
        // end frame bytes = # leds / 2 / 8 bits per byte
        // https://cpldcpu.com/2014/11/30/understanding-the-apa102-superled/
        let num_end_frames = iterator.len().div_ceil(16);
        for item in iterator {
            let item = item.into();
            match self.pixel_order {
                PixelOrder::RGB => {
                    self.spi
                        .write(&[
                            0b11100000 | u8::from(item.brightness),
                            item.red,
                            item.green,
                            item.blue,
                        ])
                        .await?
                }
                PixelOrder::RBG => {
                    self.spi
                        .write(&[
                            0b11100000 | u8::from(item.brightness),
                            item.red,
                            item.blue,
                            item.green,
                        ])
                        .await?
                }
                PixelOrder::GRB => {
                    self.spi
                        .write(&[
                            0b11100000 | u8::from(item.brightness),
                            item.green,
                            item.red,
                            item.blue,
                        ])
                        .await?
                }
                PixelOrder::GBR => {
                    self.spi
                        .write(&[
                            0b11100000 | u8::from(item.brightness),
                            item.green,
                            item.blue,
                            item.red,
                        ])
                        .await?
                }
                PixelOrder::BRG => {
                    self.spi
                        .write(&[
                            0b11100000 | u8::from(item.brightness),
                            item.blue,
                            item.red,
                            item.green,
                        ])
                        .await?
                }
                PixelOrder::BGR => {
                    self.spi
                        .write(&[
                            0b11100000 | u8::from(item.brightness),
                            item.blue,
                            item.green,
                            item.red,
                        ])
                        .await?
                }
            }
        }
        // Need an extra start frame for SK9822 to update immediately. Has no effect for APA102
        // https://cpldcpu.com/2016/12/13/sk9822-a-clone-of-the-apa102/
        self.spi.write(&[0x00, 0x00, 0x00, 0x00]).await?;
        for _ in 0..num_end_frames {
            match self.invert_end_frame {
                false => self.spi.write(&[0xFF]).await?,
                true => self.spi.write(&[0x00]).await?,
            };
        }
        Ok(())
    }
}
