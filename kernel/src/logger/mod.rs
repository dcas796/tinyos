use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use uart_16550::SerialPort;

static mut LOGGER: Option<Logger> = None;

// # Safety
// no.
pub fn logger() -> LoggerRef {
    unsafe {
        if (&LOGGER).is_none() {
            let new_logger = Logger::new();
            LOGGER = Some(new_logger);
        }
        let ptr = ((&mut LOGGER).as_mut().unwrap()) as *mut Logger;
        LoggerRef {
            ptr: NonNull::new_unchecked(ptr),
        }
    }
}

const SERIAL_PORT: u16 = 0x3f8;

pub struct Logger {
    port: SerialPort,
}

impl Logger {
    pub fn new() -> Self {
        let mut port = unsafe { SerialPort::new(SERIAL_PORT) };
        port.init();
        Self { port }
    }
}

impl Deref for Logger {
    type Target = SerialPort;

    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl DerefMut for Logger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}

pub struct LoggerRef {
    ptr: NonNull<Logger>,
}

impl Deref for LoggerRef {
    type Target = Logger;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl DerefMut for LoggerRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}
