use hil::{Driver,Callback};
use hil::i2c::I2C;
use hil::timer::*;

#[allow(dead_code)]
enum Registers {
    SensorVoltage = 0x00,
    LocalTemperature = 0x01,
    Configuration = 0x02,
    ManufacturerID = 0xFE,
    DeviceID = 0xFF
}

pub struct TMP006<I: I2C + 'static> {
    i2c: &'static mut I,
    timer: VirtualTimer,
    last_temp: Option<i16>,
    callback: Option<Callback>,
    enabled: bool
}

impl<I: I2C> TMP006<I> {
    pub fn new(i2c: &'static mut I, timer: VirtualTimer) -> TMP006<I> {
        TMP006{
            i2c: i2c,
            timer: timer,
            last_temp: None,
            callback: None,
            enabled: false
        }
    }
}

impl<I: I2C> TimerCB for TMP006<I> {
    fn fired(&mut self, _: u32) {
        let mut buf: [u8; 3] = [0; 3];

        // If not ready, wait for next timer fire
        self.i2c.read_sync(0x40, &mut buf[0..2]);
        if buf[1] & 0x80 != 0x80 {
            return;
        }

        // Now set the correct register pointer value so we can issue a read
        // to the sensor voltage register
        buf[0] = Registers::SensorVoltage as u8;
        self.i2c.write_sync(0x40, &buf[0..1]);

        // Now read the sensor reading
        self.i2c.read_sync(0x40, &mut buf[0..2]);
        //let sensor_voltage = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

        // Now move the register pointer to the die temp register
        buf[0] = Registers::LocalTemperature as u8;
        self.i2c.write_sync(0x40, &buf[0..1]);

        // Now read the 14bit die temp
        self.i2c.read_sync(0x40, &mut buf[0..2]);
        let die_temp = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

        // Shift to the right to make it 14 bits (this should be a signed shift)
        // The die temp is is in 1/32 degrees C.
        let final_temp = die_temp >> 2;
        self.last_temp = Some(final_temp);
        self.callback.take().map(|mut cb| {
            cb.schedule(final_temp as usize, 0, 0);
        });
    }
}

impl<I: I2C> Driver for TMP006<I> {
    fn subscribe(&mut self, subscribe_num: usize, mut callback: Callback) -> isize {
        match subscribe_num {
            0 /* read temperature  */ => {
                if !self.enabled {
                    return -1;
                }
                match self.last_temp {
                    Some(temp) => {
                        callback.schedule(temp as usize, 0, 0);
                    },
                    None => {
                        self.callback = Some(callback);
                    }
                }
                0
            },
            _ => -1
        }
    }

    fn command(&mut self, cmd_num: usize, _: usize) -> isize {
        match cmd_num {
            0 /* Enable sensor  */ => {
                self.i2c.enable();

                let mut buf: [u8; 3] = [0; 3];

                // Start by enabling the sensor
                let config = 0x7 << 12;
                buf[0] = Registers::Configuration as u8;
                buf[1] = ((config & 0xFF00) >> 8) as u8;
                buf[2] = (config & 0x00FF) as u8;
                self.i2c.write_sync(0x40, &buf);

                self.timer.repeat(32768);

                self.enabled = true;

                0
            },
            _ => -1
        }
    }
}

