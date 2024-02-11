//! Driver for the Synopsys Designware APB UART.
//! Doc: <https://linux-sunxi.org/images/d/d2/Dw_apb_uart_db.pdf>
//!
//! Device trees will use "snps,dw-apb-uart" for the compatibility field
use crate::drivers::{Driver, UartDriver};

/// Memory-mapped Synopsys Designware APB UART
#[repr(C, packed)]
#[derive(Debug)]
pub struct Uart {
    /// Three registers in one!
    /// RBR - Receive Buffer Register (Read only)
    /// THR - Transmit Holding Register (Write only)
    /// DLL - Divisor Latch Low(R/W)
    ///
    /// Essentially, read to receive, write to transmit UNLESS LCR[7] = 1, in which case this
    /// register will act as the Divisor Latch Low register
    pub rxtx: u32, // 0x00
    /// Divisor Latch High (if LCR[7] = 1) or Interrupt Enable Register(if LCR[7] = 0)
    pub dlh_ier: u32, // 0x04
    /// Interrupt Identification Register (if read) or FIFO control register (if written)
    pub irr_fcr: u32, // 0x08
    /// Line Control Register (read/write)
    pub lcr: u32, // 0x0C
    /// Modem Control Register (read/write)
    pub mcr: u32, // 0x10
    /// Line Status Register (read only)
    pub lsr: u32, // 0x14
    /// Modem Status Register (read only)
    pub msr: u32, // 0x18
}

/// UART driver for the Synopsys Designware APB Uart
#[derive(Debug)]
pub struct DwApbUartDriver {
    uart: *mut Uart,
}

unsafe impl Sync for DwApbUartDriver {}

impl DwApbUartDriver {
    /// Initialize the driver
    pub fn new(base_address: *mut u8) -> Self {
        let _driver = Self {
            uart: base_address as *mut Uart,
        };

        todo!("Initialize");
    }
}

impl Driver for DwApbUartDriver {}

impl UartDriver for DwApbUartDriver {
    fn next_byte(&self) -> u8 {
        todo!()
    }

    fn send_byte(&self, byte: u8) {
        unsafe {
            (*(self.uart)).rxtx = byte as u32;
        }
    }
}
