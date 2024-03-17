//! Drivers and driver accessories
use crate::{prelude::*, KernelResult};
use alloc::boxed::Box;
use core::{cell::RefCell, fmt::Debug, time::Duration};
use fdt::{node::FdtNode, Fdt};
use spin::Mutex;
pub mod clint_timer;
pub mod ns16550;
pub mod sifive_uart;
use utf8_parser::Utf8Parser;

type DriverBox<T> = Mutex<RefCell<Option<Box<T>>>>;

/// Collection of initialized drivers
#[derive(Debug)]
pub struct Drivers {
    /// The UART driver
    pub uart: DriverBox<dyn UartDriver>,
    /// The timer driver
    pub timer: DriverBox<dyn TimerDriver>,
}

impl Drivers {
    fn init_uart(&self, node: &FdtNode) -> KernelResult<()> {
        let uart = self.uart.lock();
        let mut uart = uart.borrow_mut();

        // Don't reinit
        if uart.is_some() {
            return Ok(());
        }

        *uart = ns16550::Ns16550Driver::maybe_from_node(node)
            .transpose()
            .or_else(|| sifive_uart::SifiveUartDriver::maybe_from_node(node).transpose())
            .transpose()?;

        Ok(())
    }

    fn init_timer(&self, tree: &Fdt, node: &FdtNode) -> KernelResult<()> {
        let timer = self.timer.lock();
        let mut timer = timer.borrow_mut();

        // Don't reinit
        if timer.is_some() {
            return Ok(());
        }

        *timer = clint_timer::ClintTimerDriver::maybe_from_node(tree, node)?;

        if let Some(_timer) = &mut *timer {
            println!("[Timer driver loaded]");
        }

        KernelResult::Ok(())
    }

    pub fn init(&self, fdt: &Fdt) -> KernelResult<()> {
        let chosen = fdt.chosen();
        if let Some(stdout) = chosen.stdout() {
            self.init_uart(&stdout.node())?;
        }

        for node in fdt.all_nodes() {
            self.init_uart(&node)?;
            self.init_timer(fdt, &node)?;
        }

        Ok(())
    }
}

/// Global object that keeps track of initialized drivers
pub static DRIVERS: Drivers = Drivers {
    uart: Mutex::new(RefCell::new(None)),
    timer: Mutex::new(RefCell::new(None)),
};

/// Driver for a "disk.' This can be NOR flash, an SSD, a hard drive, or just RAM.
pub trait TimerDriver: Debug + Send {
    fn set_alarm(&mut self, hart: HartId, duration: Duration);
}

/// A UART/serial driver
pub trait UartDriver: Debug + Send {
    /// Read the next byte out of the UART
    fn next_byte(&mut self) -> u8;

    /// Write a byte to the UART
    fn send_byte(&mut self, byte: u8);

    /// Read the next character from the UART
    fn next_char(&mut self) -> char {
        let mut parser = Utf8Parser::default();
        loop {
            if let Some(c) = parser
                .push(self.next_byte())
                .unwrap_or(Some(char::REPLACEMENT_CHARACTER))
            {
                return c;
            }
        }
    }

    /// Send a string to the UART
    fn send_str(&mut self, s: &str) {
        for byte in s.as_bytes() {
            self.send_byte(*byte)
        }
    }
}
