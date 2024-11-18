use once_cell::sync::Lazy;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::RwLock;
use std::thread;

use rmodbus::{
    server::{storage::ModbusStorageFull, ModbusFrame},
    ModbusFrameBuf, ModbusProto,
};

// pub for README example
pub static CONTEXT: Lazy<RwLock<ModbusStorageFull>> = Lazy::new(<_>::default);

pub fn tcpserver(unit: u8, listen: &str) {
    let listener = TcpListener::bind(listen).unwrap();
    println!("listening started, ready to accept");
    for stream in listener.incoming() {
        thread::spawn(move || {
            println!("client connected");
            let mut stream = stream.unwrap();
            loop {
                let mut buf: ModbusFrameBuf = [0; 256];
                let mut response = Vec::new(); // for nostd use FixedVec with alloc [u8;256]
                if stream.read(&mut buf).unwrap_or(0) == 0 {
                    return;
                }
                let mut frame = ModbusFrame::new(unit, &buf, ModbusProto::TcpUdp, &mut response);
                if frame.parse().is_err() {
                    println!("server error");
                    return;
                }
                if frame.processing_required {
                    let result = if frame.readonly {
                        frame.process_read(&*CONTEXT.read().unwrap())
                    } else {
                        frame.process_write(&mut *CONTEXT.write().unwrap())
                    };
                    if result.is_err() {
                        println!("frame processing error");
                        return;
                    }
                }
                if frame.response_required {
                    frame.finalize_response().unwrap();
                    println!("{:x?}", response.as_slice());
                    if stream.write(response.as_slice()).is_err() {
                        return;
                    }
                }
            }
        });
    }
}
