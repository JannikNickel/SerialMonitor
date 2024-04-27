mod serial_reader;

use std::{str::FromStr, time::SystemTime};
use std::time::Duration;
use serial_reader::{SerialConfig, SerialReader, StartMode};

fn main() {
    let config = SerialConfig {
        port: String::from_str("COM3").unwrap(),
        baud_rate: 9600,
        data_bits: 8,
        parity: serial_reader::Parity::None,
        stop_bits: 1,
        timeout: Duration::from_millis(1)
    };
    let start_mode = StartMode::Delay(Duration::from_millis(1000));

    let mut port = SerialReader::new(config);
    match port.open(true) {
        Ok(_) =>  {
            println!("Opened port!");
            if port.begin_read(start_mode).is_ok() {
                println!("Beginning to read!");
                fwd_std_out(&mut port);
            }
        },
        Err(e) => eprintln!("{}", e)
    }
}

fn fwd_std_out(reader: &mut SerialReader) {
    let start = SystemTime::now();
    loop {
        if let Some(line_res) = reader.get_line() {
            match line_res {
                Ok(line) => println!("[{:.2}] > {}", SystemTime::now().duration_since(start).unwrap().as_secs_f32(), line),
                Err(e) => eprintln!("{}", e)
            }
        }
    }
}
