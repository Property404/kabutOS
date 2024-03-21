//! Drivers and driver accessories
use crate::prelude::*;
use alloc::{boxed::Box, collections::BTreeSet};
use core::{
    fmt::{self, Debug, Write},
    time::Duration,
};
use fdt::{node::FdtNode, Fdt};
use spin::Mutex;
pub mod clint_timer;
pub mod ns16550;
pub mod plic;
use utf8_parser::Utf8Parser;

type DriverBox<T> = Mutex<Option<Box<T>>>;

static LOADERS: &[DriverLoader] = &[ns16550::LOADER, clint_timer::LOADER, plic::LOADER];

/// Collection of initialized drivers
#[derive(Debug)]
pub struct Drivers {
    /// The UART driver
    pub uart: DriverBox<dyn UartDriver>,
    /// The timer driver
    pub timer: DriverBox<dyn TimerDriver>,
    /// The IC driver
    pub ic: DriverBox<dyn InterruptControllerDriver>,
}

impl Drivers {
    fn load(&self, res: LoadResult) {
        match res {
            LoadResult::Uart(dev) => {
                (*self.uart.lock()).get_or_insert(dev);
            }
            LoadResult::Timer(dev) => {
                (*self.timer.lock()).get_or_insert(dev);
            }
            LoadResult::InterruptController(dev) => {
                (*self.ic.lock()).get_or_insert(dev);
            }
        }
    }

    fn init_node(
        &self,
        picked: &mut BTreeSet<&'static str>,
        fdt: &'static Fdt<'static>,
        node: FdtNode<'static, 'static>,
    ) -> KernelResult<()> {
        if picked.contains(node.name) {
            return Ok(());
        }
        for loader in LOADERS {
            let info = LoadInfo {
                fdt: *fdt,
                node,
            };
            if let Some(dev) = loader.maybe_load(&info)? {
                self.load(dev);
                assert!(picked.insert(node.name));
            }
        }
        Ok(())
    }

    pub fn init(&self, fdt: &'static Fdt<'static>) -> KernelResult<()> {
        let mut picked = BTreeSet::new();
        let chosen = fdt.chosen();

        if let Some(stdout) = chosen.stdout() {
            let node = stdout.node();
            self.init_node(&mut picked, fdt, node)?;
        }

        for node in fdt.all_nodes() {
            self.init_node(&mut picked, fdt, node)?;
        }

        Ok(())
    }
}

/// Global object that keeps track of initialized drivers
pub static DRIVERS: Drivers = Drivers {
    uart: Mutex::new(None),
    timer: Mutex::new(None),
    ic: Mutex::new(None),
};

/// Interrupt controller driver
pub trait InterruptControllerDriver: Debug + Send {
    fn enable(&mut self, interrupt: InterruptId);
    // Get this interrupt and claim it
    fn next(&mut self) -> Option<InterruptId>;
}

/// Driver for a CPU timer
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

impl Write for dyn UartDriver {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.send_str(s);
        Ok(())
    }
}

struct LoadInfo {
    fdt: Fdt<'static>,
    node: FdtNode<'static, 'static>,
}

enum LoadResult {
    Uart(Box<dyn UartDriver>),
    InterruptController(Box<dyn InterruptControllerDriver>),
    Timer(Box<dyn TimerDriver>),
}

#[derive(Copy, Clone)]
struct DriverLoader {
    compatible: &'static str,
    /// Perform some arbitrary action with this node
    load: fn(info: &LoadInfo) -> KernelResult<LoadResult>,
}

impl DriverLoader {
    fn maybe_load(&self, info: &LoadInfo) -> KernelResult<Option<LoadResult>> {
        let Some(compatible) = info.node.compatible() else {
            return Ok(None);
        };
        if !compatible.all().any(|v| v == self.compatible) {
            return Ok(None);
        }
        (self.load)(info).map(Some)
    }
}
