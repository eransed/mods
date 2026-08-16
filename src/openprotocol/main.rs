use std::net::Incoming;

use crate::{
    client::session,
    core::{Mid, get_mid},
    mid_0002::mid_parse_0002_rev6,
    mid_0004::{mid_parse_0004, print_header},
};

extern crate num;
#[macro_use]
extern crate num_derive;

mod client;
mod core;
mod mid_0002;
mod mid_0004;

fn read_mid() -> Result<(), String> {
    let raw_data = "025200022345987668123894761234akjshfgjashmnbmnbmnbmnbmnbmnbmnbmnbmbnnmbmbmnbhjgjhgjhgjhgjhbmnbvnnmbnvnbvnbvnbvbnvnbnvnvnvnbvnbvdfnbsadfbasxbzcmvbnzx,bcvjksagfjkashgdfmxncvbxzm,cvz,mxbv,mzxbcvzxmbvcjsfdhakshfkahsldfasdfasdfasdfasdfasdfasdfasdfasdfasdfas";
    let mid_number = get_mid(raw_data)?;
    match mid_number {
        2 => {
            let m2 = mid_parse_0002_rev6(raw_data)?;
            let name = m2.r5.r4.r3.r2.r1.controller_name;
            println!("Name: {}", name);
        }
        4 => {
            let m4 = mid_parse_0004(raw_data)?;
            print_header(m4);
            println!("header.name: {}", m4.header.name());
        }
        _ => {
            return Err(format!("Mid {} not supported", mid_number));
        }
    }
    return Ok(());
}

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("192.168.0.47:4545").await?;
    let (mut rd, mut wr) = io::split(socket);
    let mut quit = false;
    let keep_alive_timout_ms = 5000;
    let mut last_ka = std::time::Instant::now();

    // Write data in the background
    tokio::spawn(async move {
        wr.write_all(b"00200001006         ").await?;
        wr.write_all(b"\0").await?;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(keep_alive_timout_ms));
            if quit == true {
                println!("Quit");
                break;
            }

            last_ka = std::time::Instant::now();
            wr.write_all(b"00209999001         ").await?;
            wr.write_all(b"\0").await?;
        }

        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0; 4096];

    loop {
        let n = rd.read(&mut buf).await?;

        let elap = last_ka.elapsed();

        if n == 0 {
            println!("Recived 0");
            quit = true;
            break;
        }

        let s = String::from_utf8(buf[0..n].to_vec()).unwrap();

        println!("Recived {}({})[{:.1?}] bytes: >>>{}<<<", n, s.len(), elap,  s);
    }

    Ok(())
}

// fn main() {
//     println!("oprs");

//     let _ = read_mid().unwrap();

//     match session() {
//         Ok(_) => {
//             println!("OK!");
//         }
//         Err(e) => {
//             println!("Error {}", e);
//         }
//     }
// }
