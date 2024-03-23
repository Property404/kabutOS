//! Drivers and driver accessories
use crate::{interrupts, prelude::*, process::ProcessState, scheduler};
use alloc::{
    collections::{BTreeSet, VecDeque},
    sync::Arc,
};
use core::{
    fmt::{self, Debug, Write},
    time::Duration,
};
use fdt::{node::FdtNode, Fdt};
use spin::{Mutex, RwLock};
pub mod clint_timer;
pub mod ns16550;
pub mod plic;
use utf8_parser::Utf8Parser;

#[derive(Debug)]
pub struct Driver<T: Send + ?Sized> {
    pub info: DriverInfo,
    /// Hardware coupling
    pub coupling: Box<T>,
}

struct WrappedDriver {
    info: DriverInfo,
    coupling: LoadResult,
}

#[derive(Clone, Debug)]
pub struct DriverInfo {
    pub interrupt_parent: Option<u32>,
    pub interrupts: Vec<InterruptId>,
}

type DriverBox2<T> = RwLock<Option<Arc<Mutex<T>>>>;
type DriverBox<T> = Mutex<Option<T>>;

static LOADERS: &[DriverLoader] = &[ns16550::LOADER, clint_timer::LOADER, plic::LOADER];

/// Collection of initialized drivers
#[derive(Debug)]
pub struct Drivers {
    /// The UART driver
    pub uart: DriverBox2<Driver<dyn UartDriver>>,
    /// The timer driver
    pub timer: DriverBox<Box<dyn TimerDriver>>,
    /// The IC driver
    pub ic: DriverBox<Box<dyn InterruptControllerDriver>>,
}

impl Drivers {
    fn load(&self, WrappedDriver { info, coupling }: WrappedDriver) {
        match coupling {
            LoadResult::Uart(coupling) => {
                let driver = Arc::new(Mutex::new(Driver {
                    info: info.clone(),
                    coupling,
                }));
                (*self.uart.write()).get_or_insert(driver.clone());
                if let Some(int_id) = info.interrupts.first() {
                    let driver = driver.clone();
                    // TODO: this should be a ringbuffer
                    let uart_buffer = Arc::new(Mutex::new((Utf8Parser::new(), VecDeque::new())));
                    interrupts::register_handler(*int_id, move |int_id| {
                        // Push next chars into buffer
                        let mut driver = driver.lock();
                        let mut buffer = uart_buffer.lock();
                        while let Some(byte) = driver.coupling.next_byte() {
                            if let Some(ch) = buffer.0.push(byte)? {
                                buffer.1.push_back(ch);
                            }
                        }

                        // And send to process
                        scheduler::on_interrupt(int_id, |process| {
                            if let Some(c) = buffer.1.pop_front() {
                                process.frame.as_mut().set_return_value(&Ok(c as usize));
                                process.state = ProcessState::Ready;
                            }
                            Ok(())
                        })?;
                        Ok(())
                    });
                }
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
        interrupt_registrations: &mut Vec<(u32, InterruptId)>,
        picked: &mut BTreeSet<&'static str>,
        fdt: &'static Fdt<'static>,
        node: FdtNode<'static, 'static>,
    ) -> KernelResult<()> {
        if picked.contains(node.name) {
            return Ok(());
        }
        for loader in LOADERS {
            let mut ctx = LoadContext { fdt: *fdt, node };
            if let Some(dev) = loader.maybe_load(&mut ctx)? {
                if let Some(interrupt_parent) = &dev.info.interrupt_parent {
                    for interrupt in &dev.info.interrupts {
                        interrupt_registrations.push((*interrupt_parent, *interrupt));
                    }
                }
                self.load(dev);
                assert!(picked.insert(node.name));
            }
        }
        Ok(())
    }

    pub fn init(&self, fdt: &'static Fdt<'static>) -> KernelResult<()> {
        let mut picked = BTreeSet::new();
        let mut interrupt_registrations = Vec::new();
        let chosen = fdt.chosen();

        if let Some(stdout) = chosen.stdout() {
            let node = stdout.node();
            self.init_node(&mut interrupt_registrations, &mut picked, fdt, node)?;
        }

        for node in fdt.all_nodes() {
            self.init_node(&mut interrupt_registrations, &mut picked, fdt, node)?;
        }

        // Register interrupts
        if let Some(ic) = &mut *self.ic.lock() {
            for (phandle, id) in interrupt_registrations {
                if ic.phandle() == phandle {
                    ic.enable(id);
                }
            }
        }

        Ok(())
    }
}

/// Global object that keeps track of initialized drivers
pub static DRIVERS: Drivers = Drivers {
    uart: RwLock::new(None),
    timer: Mutex::new(None),
    ic: Mutex::new(None),
};

/// Interrupt controller driver
pub trait InterruptControllerDriver: Debug + Send {
    fn phandle(&self) -> u32;
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
    fn next_byte(&mut self) -> Option<u8>;

    /// Read the next byte out of UART (spin blocks)
    fn spin_until_next_byte(&mut self) -> u8 {
        loop {
            if let Some(byte) = self.next_byte() {
                return byte;
            }
        }
    }

    /// Write a byte to the UART
    fn send_byte(&mut self, byte: u8);

    /// Read the next character from the UART (spins)
    fn spin_until_next_char(&mut self) -> char {
        let mut parser = Utf8Parser::default();
        loop {
            if let Some(c) = parser
                .push(self.spin_until_next_byte())
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

struct LoadContext {
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
    load: fn(info: &LoadContext) -> KernelResult<LoadResult>,
}

impl DriverLoader {
    fn maybe_load(&self, ctx: &mut LoadContext) -> KernelResult<Option<WrappedDriver>> {
        let Some(compatible) = ctx.node.compatible() else {
            return Ok(None);
        };
        if !compatible.all().any(|v| v == self.compatible) {
            return Ok(None);
        }

        let interrupt_parent: Option<u32> = ctx
            .node
            .property("interrupt-parent")
            .map(|interrupt_parent| {
                interrupt_parent
                    .as_usize()
                    .ok_or(KernelError::Generic("Invalid phandle"))
            })
            .transpose()?
            .map(TryInto::try_into)
            .transpose()?;

        let mut interrupts = Vec::new();
        for interrupt in ctx.node.interrupts().into_iter().flatten() {
            let interrupt = InterruptId::try_from(interrupt)?;
            interrupts.push(interrupt);
        }

        let info = DriverInfo {
            interrupt_parent,
            interrupts,
        };
        let coupling = (self.load)(ctx)?;
        Ok(Some(WrappedDriver { info, coupling }))
    }
}
