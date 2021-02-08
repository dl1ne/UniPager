use serial::{self, SerialPort};
use std::{thread, time, mem};

use config::Config;
use transmitter::Transmitter;

pub struct RFM69Transmitter {
    serial: Box<serial::SerialPort>
}

impl RFM69Transmitter {
    pub fn new(config: &Config) -> RFM69Transmitter {
        info!("Initializing RFM69 transmitter...");

        let mut serial = serial::open(&config.rfm69.port).expect(
            "Unable to open serial port"
        );

        serial
            .configure(&serial::PortSettings {
                baud_rate: serial::BaudRate::Baud38400,
                char_size: serial::CharSize::Bits8,
                parity: serial::Parity::ParityNone,
                stop_bits: serial::StopBits::Stop1,
                flow_control: serial::FlowControl::FlowNone
            })
            .expect("Unable to configure serial port");

        RFM69Transmitter { serial: Box::new(serial) }
    }
}

impl Transmitter for RFM69Transmitter {
    fn send(&mut self, gen: &mut Iterator<Item = u32>) {
        for word in gen {

            let bytes = [
                ((word & 0xff000000) >> 24) as u8,
                ((word & 0x00ff0000) >> 16) as u8,
                ((word & 0x0000ff00) >> 8) as u8,
                (word & 0x000000ff) as u8,
            ];

            // Retry a couple of times to send data
            for _ in 0..5 {
               if (!(*self.serial).write_all(&bytes).is_err()) {
                  break;
               }
               thread::sleep(time::Duration::from_millis(50));
            }
        }

        // Send End of Transmission packet
        let eot = [0x17 as u8];
        if (*self.serial).write_all(&eot).is_err() {
            error!("Unable to send end of transmission byte");
            return;
        }

        if (*self.serial).flush().is_err() {
            error!("Unable to flush serial port");
        }

        // Wait a little bit
        thread::sleep(time::Duration::from_millis(500));
        
        // Lets check for reply from radio, if packet is send
        loop {
            let mut buf = [0 as u8];
            (*self.serial).read(&mut buf);
            //info!("Received: {:?}", buf);
            if buf == eot {
               break;
            }
        }


    }
}
