#![allow(dead_code)]

use embsys::crates::display_interface;
use embsys::crates::embassy_embedded_hal;
use embsys::crates::embassy_rp;
use embsys::crates::embassy_sync;
use embsys::crates::embassy_time;
use embsys::drivers::hardware::HWController;
use embsys::exts::std;

use std::cell::RefCell;
use std::vec::Vec;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Output;
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::Blocking;
use embassy_rp::spi::ClkPin;
use embassy_rp::spi::Config;
use embassy_rp::spi::Instance as SpiInstance;
use embassy_rp::spi::MosiPin;
use embassy_rp::spi::Spi;
use embassy_rp::Peripheral;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Delay;

use ssd1351::builder::Builder;
use ssd1351::mode::GraphicsMode;
use ssd1351::prelude::*;
use ssd1351::properties::DisplayRotation;

use display_interface::WriteOnlyDataCommand;

pub type SPIMode<'a> = SPIInterface<
    SpiDeviceWithConfig<'static, NoopRawMutex, Spi<'static, SPI0, Blocking>, Output<'a>>,
    Output<'a>,
>;

static mut BUFFER: StaticBuffer<u8, 32768> = StaticBuffer::new();
static mut SPI_BUS: Option<SPIBus<embassy_rp::peripherals::SPI0>> = None;

pub struct SPIBus<T>
where
    T: SpiInstance + 'static,
{
    spi_bus: Mutex<NoopRawMutex, RefCell<Spi<'static, T, Blocking>>>,
    config: Config,
}

impl<T> SPIBus<T>
where
    T: SpiInstance + 'static,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'static,
        clk: impl ClkPin<T>,
        mosi: impl MosiPin<T>,
    ) -> Self {
        let mut config: Config = Config::default();
        config.frequency = 40_000_000;

        let spi: Spi<'_, T, Blocking> =
            Spi::new_blocking_txonly(peripheral, clk, mosi, config.clone());
        let spi_bus: Mutex<NoopRawMutex, RefCell<Spi<'_, T, Blocking>>> =
            Mutex::new(RefCell::new(spi));

        Self { spi_bus, config }
    }
}

pub struct SPIDevice;

impl SPIDevice {
    pub fn create<'a, T>(
        bus: &'a SPIBus<T>,
        cs: Output<'a>,
    ) -> SpiDeviceWithConfig<'a, NoopRawMutex, Spi<'static, T, Blocking>, Output<'a>>
    where
        T: SpiInstance + 'static,
    {
        let device: SpiDeviceWithConfig<'_, NoopRawMutex, Spi<'_, T, Blocking>, Output<'a>> =
            SpiDeviceWithConfig::new(&bus.spi_bus, cs, bus.config.clone());
        device
    }
}

pub struct SPIDisplayInterface;

impl SPIDisplayInterface {
    pub fn create<'a, T>(
        device: SpiDeviceWithConfig<'a, NoopRawMutex, Spi<'static, T, Blocking>, Output<'a>>,
        dc: Output<'a>,
    ) -> SPIInterface<
        SpiDeviceWithConfig<'a, NoopRawMutex, Spi<'static, T, Blocking>, Output<'a>>,
        Output<'a>,
    >
    where
        T: SpiInstance + 'static,
    {
        let interface: SPIInterface<
            SpiDeviceWithConfig<'a, NoopRawMutex, Spi<'_, T, Blocking>, Output<'a>>,
            Output<'a>,
        > = SPIInterface::new(device, dc);
        interface
    }
}

pub struct StaticBuffer<T, const SIZE: usize> {
    v: Vec<T>,
}

impl<T, const SIZE: usize> StaticBuffer<T, SIZE>
where
    T: Default,
{
    pub const fn new() -> Self {
        let v: Vec<T> = Vec::new();
        Self { v }
    }

    pub fn init(&mut self) {
        for _ in 0..SIZE {
            self.v.push(T::default());
        }
    }

    pub fn as_mut(&mut self) -> &mut Vec<T> {
        &mut self.v
    }
}

pub struct GraphicsDisplay<T>
where
    T: WriteOnlyDataCommand,
{
    display: GraphicsMode<T>,
}

impl<T> GraphicsDisplay<T>
where
    T: WriteOnlyDataCommand,
{
    pub fn new(display: GraphicsMode<T>) -> Self {
        Self { display }
    }

    pub fn get(&mut self) -> &mut GraphicsMode<T> {
        &mut self.display
    }

    pub fn refresh(&mut self) {
        self.display.flush();
        self.display.clear(false);
    }
}

pub fn setup_display<'a>() -> GraphicsDisplay<SPIMode<'a>> {
    unsafe { BUFFER.init() };

    let clk = HWController::pac().PIN_2;
    let mosi = HWController::pac().PIN_3;
    let spi_bus: SPIBus<SPI0> = SPIBus::new(HWController::pac().SPI0, clk, mosi);
    unsafe { SPI_BUS = Some(spi_bus) };

    let cs: Output<'_> = Output::new(HWController::pac().PIN_5, Level::Low);
    let dc: Output<'_> = Output::new(HWController::pac().PIN_6, Level::Low);
    let mut rst: Output<'_> = Output::new(HWController::pac().PIN_7, Level::Low);

    let spi_device = SPIDevice::create(unsafe { SPI_BUS.as_ref().unwrap() }, cs);
    let spi_interface = SPIDisplayInterface::create(spi_device, dc);

    let mut display: GraphicsMode<_> = Builder::new()
        .with_rotation(DisplayRotation::Rotate90)
        .connect_interface(spi_interface, unsafe { BUFFER.as_mut() })
        .into();

    display.reset(&mut rst, &mut Delay).unwrap();
    display.init().unwrap();

    let display: GraphicsDisplay<SPIMode<'a>> = GraphicsDisplay::new(display);
    display
}
