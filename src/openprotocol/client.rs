use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;
use std::{thread, time};

pub fn session() -> std::io::Result<()> {
    let ip = "192.168.0.47";
    let port = 4545;
    let addr = format!("{}:{}", ip, port);

    let mut stream = TcpStream::connect(addr)?;

    {
        let mid0001 = "00200001006         ";
        let mut buffer = [0; 4096];
        let now = Instant::now();
        stream.write_all(mid0001.as_bytes())?;
        stream.write_all(&[0x00])?;
        let n = stream.read(&mut buffer[..])?;
        let elapsed = now.elapsed();
        let s = String::from_utf8(buffer.to_vec()).unwrap();
        println!("Read {} bytes [{:.1?}]: >>>{}<<<", n, elapsed, s);
    }

    {
        let mid0216 = "00230216001000000000020";
        let mid0218 = "00200218001000000000";
        let mut buffer = [0; 4096];
        let now = Instant::now();
        stream.write_all(mid0216.as_bytes())?;
        stream.write_all(&[0x00])?;
        let n = stream.read(&mut buffer[..])?;
        let elapsed = now.elapsed();
        let s = String::from_utf8(buffer.to_vec()).unwrap();
        println!("Read {} bytes [{:.1?}]: >>>{}<<<", n, elapsed, s);
    }

    let keep_alive_timout_ms = 5000;

    loop {
        let loop_start = Instant::now();
        let mut buffer = [0; 4096];
        let mid9999 = "00209999001         ";
        let bytes = mid9999.as_bytes();
        thread::sleep(time::Duration::from_millis(keep_alive_timout_ms));

        let now = Instant::now();

        stream.write_all(bytes)?;
        stream.write_all(&[0x00])?;

        loop {
            let n = stream.read(&mut buffer[..])?;
            
            let elapsed = now.elapsed();
            
            let s = String::from_utf8(buffer.to_vec()).unwrap();
            
            let loop_elapsed = loop_start.elapsed();
            println!(
                "Recived {} bytes [KARTT: {:.1?} / KA: {:.0?}]: >>>{}<<<",
                n, elapsed, loop_elapsed, s
            );
        }
    }

    // stream.write_all(mid0003.as_bytes())?;
    // stream.write_all(&[0x00])?;

    // let mut buffer = [0; 4096];
    // let n = stream.read(&mut buffer[..])?;
    // let s = String::from_utf8(buffer.to_vec()).unwrap();

    // println!("Read {} bytes:", n);
    // println!("   >>>> {} <<<<", s);

    Ok(())
} // the stream is closed here
