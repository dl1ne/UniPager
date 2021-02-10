use serial::{self, SerialPort}; 
use std::{thread, time};
use std::io::Write;

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


        
        let cfg = [0x18 as u8];
        let eot = [0x17 as u8];
        let freq = config.rfm69.freq;
        let power = [config.rfm69.output_level as u8];

        let b1 = [ (freq >> 24) as u8 ];
        let b2 = [ (freq >> 16) as u8 ];
        let b3 = [ (freq >> 8) as u8 ];
        let b4 = [ freq as u8 ];

        let all = [cfg, b1, b2, b3, b4, cfg, power, eot];

	for item in all.iter() {
            for _ in 0..5 {
   	        if !serial.write_all(item).is_err() {
		    break;
		}
		thread::sleep(time::Duration::from_millis(10));
	    }
	}

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

            for _ in 0..5 {
               if !(*self.serial).write_all(&bytes).is_err() {
                  break;
               }
               thread::sleep(time::Duration::from_millis(10));
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

        loop {
            let mut buf = [0 as u8];
            (*self.serial).read(&mut buf);
            if buf == eot {
               break;
            }
            thread::sleep(time::Duration::from_millis(10));
        }


    }
}
