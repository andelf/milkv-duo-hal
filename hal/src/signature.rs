#![allow(non_camel_case_types)]
use core::ptr;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChipPackage {
    Unknown,
    QFN = 0b001,
    BGA = 0b010,
}
impl ChipPackage {
    fn from_u32(value: u32) -> ChipPackage {
        match value {
            0b001 => ChipPackage::QFN,
            0b010 => ChipPackage::BGA,
            _ => ChipPackage::Unknown,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DDR_Type {
    DDR2 = 2,
    DDR3 = 3,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum DDR_Capacity {
    Unknown = 0b000,
    _4G = 0b100,
    _2G = 0b011,
    _1G = 0b010,
    _512M = 0b001,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[allow(non_camel_case_types)]
pub enum DDR_Vendor {
    /*
    #define DDR_VENDOR_UNKNOWN		0b00000
    #define DDR_VENDOR_NY_4G		0b00001
    #define DDR_VENDOR_NY_2G		0b00010
    #define DDR_VENDOR_ESMT_1G		0b00011
    #define DDR_VENDOR_ESMT_512M_DDR2	0b00100
    #define DDR_VENDOR_ETRON_1G		0b00101
    #define DDR_VENDOR_ESMT_2G		0b00110
    #define DDR_VENDOR_PM_2G		0b00111
    #define DDR_VENDOR_PM_1G		0b01000
    #define DDR_VENDOR_ETRON_512M_DDR2	0b01001
    #define DDR_VENDOR_ESMT_N25_1G		0b01010 */
    Unknown,
    NY_4G = 0b00001,
    NY_2G = 0b00010,
    ESMT_1G = 0b00011,
    ESMT_512M_DDR2 = 0b00100,
    ETRON_1G = 0b00101,
    ESMT_2G = 0b00110,
    PM_2G = 0b00111,
    PM_1G = 0b01000,
    ETRON_512M_DDR2 = 0b01001,
    ESMT_N25_1G = 0b01010,
}

impl DDR_Vendor {
    fn from_u32(value: u32) -> DDR_Vendor {
        match value {
            0b00000 => DDR_Vendor::Unknown,
            0b00001 => DDR_Vendor::NY_4G,
            0b00010 => DDR_Vendor::NY_2G,
            0b00011 => DDR_Vendor::ESMT_1G,
            0b00100 => DDR_Vendor::ESMT_512M_DDR2,
            0b00101 => DDR_Vendor::ETRON_1G,
            0b00110 => DDR_Vendor::ESMT_2G,
            0b00111 => DDR_Vendor::PM_2G,
            0b01000 => DDR_Vendor::PM_1G,
            0b01001 => DDR_Vendor::ETRON_512M_DDR2,
            0b01010 => DDR_Vendor::ESMT_N25_1G,
            _ => DDR_Vendor::Unknown,
        }
    }
    pub fn ddr_type(&self) -> DDR_Type {
        match self {
            DDR_Vendor::ESMT_512M_DDR2 | DDR_Vendor::ETRON_512M_DDR2 => DDR_Type::DDR2,
            _ => DDR_Type::DDR3,
        }
    }

    pub fn ddr_capacity(&self) -> DDR_Capacity {
        match self {
            DDR_Vendor::NY_4G => DDR_Capacity::_4G,
            DDR_Vendor::NY_2G | DDR_Vendor::ESMT_2G | DDR_Vendor::PM_2G => DDR_Capacity::_2G,
            DDR_Vendor::ESMT_1G | DDR_Vendor::ETRON_1G | DDR_Vendor::PM_1G | DDR_Vendor::ESMT_N25_1G => {
                DDR_Capacity::_1G
            }
            DDR_Vendor::ESMT_512M_DDR2 | DDR_Vendor::ETRON_512M_DDR2 => DDR_Capacity::_512M,
            DDR_Vendor::Unknown => DDR_Capacity::Unknown,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ChipInfo {
    pub ddr_vendor: DDR_Vendor,
    pub ddr_capacity: DDR_Capacity,
    pub ddr_type: DDR_Type,
    pub package: ChipPackage,
    pub chip_id: u32,
}

pub fn read_chip_info() -> ChipInfo {
    let conf_info = unsafe { ptr::read_volatile(0x03000004 as *const u32) };
    let efuse_leakage = unsafe { ptr::read_volatile(0x03050108 as *const u32) };
    //let ftsn3 = ptr::read_volatile(0x0305010c as *const u32);
    //let ftsn4 = ptr::read_volatile(0x03050110 as *const u32);

    // pkg_type = FIELD_GET(conf_info, 30, 28);
    let pkg_type = (conf_info >> 28) & 0x7;

    let mut ddr_vendor = DDR_Vendor::from_u32(0);
    let mut ddr_capacity = DDR_Capacity::Unknown;
    let mut ddr_type = DDR_Type::DDR3;
    let mut package = ChipPackage::Unknown;
    let mut chip_id = 0;

    match pkg_type {
        0x0 => {
            // BGA 10x10, SIP 2Gb DDR3
            ddr_vendor = DDR_Vendor::NY_2G;
            ddr_capacity = DDR_Capacity::_2G;
            package = ChipPackage::BGA;
        }
        0x1 => {
            // BGA 10x10, SIP 4Gb DDR3
            ddr_vendor = DDR_Vendor::NY_4G;
            ddr_capacity = DDR_Capacity::_4G;
            package = ChipPackage::BGA;
        }
        0x2 => {
            //BGA 10x10, SIP 1Gb DDR3
            ddr_vendor = DDR_Vendor::ESMT_1G;
            ddr_capacity = DDR_Capacity::_1G;
            package = ChipPackage::BGA;
        }
        0x4 => {
            ddr_vendor = DDR_Vendor::from_u32((efuse_leakage >> 21) & 0x1F);
            ddr_capacity = ddr_vendor.ddr_capacity();
            ddr_type = ddr_vendor.ddr_type();
            package = ChipPackage::from_u32((efuse_leakage >> 29) & 0x7);
        }
        0x5 => {
            // QFN9x9, SIP 2Gb DDR3
            ddr_vendor = DDR_Vendor::NY_2G;
            ddr_capacity = DDR_Capacity::_2G;
            package = ChipPackage::QFN;
        }
        0x6 => {
            // QFN9x9, SIP 1Gb DDR3
            ddr_vendor = DDR_Vendor::ESMT_1G;
            ddr_capacity = DDR_Capacity::_1G;
            package = ChipPackage::QFN;
        }
        0x7 => {
            // QFN9x9, SIP 512Mb DDR2
            ddr_vendor = DDR_Vendor::ESMT_512M_DDR2;
            ddr_capacity = DDR_Capacity::_512M;
            ddr_type = DDR_Type::DDR2;
            package = ChipPackage::QFN;
        }
        _ => {}
    }

    chip_id = match (ddr_capacity, package) {
        (DDR_Capacity::_512M, ChipPackage::QFN) => 0x1810c,
        (DDR_Capacity::_512M, ChipPackage::BGA) => 0x1810f,
        (DDR_Capacity::_1G, ChipPackage::QFN) => 0x1811c,
        (DDR_Capacity::_1G, ChipPackage::BGA) => 0x1811f,
        (DDR_Capacity::_2G, ChipPackage::QFN) => 0x1812c,
        (DDR_Capacity::_2G, ChipPackage::BGA) => 0x1812f,
        (DDR_Capacity::_4G, ChipPackage::BGA) => 0x1813f,
        _ => 0,
    };

    let chip_info = ChipInfo {
        ddr_vendor,
        ddr_capacity,
        ddr_type,
        package,
        chip_id,
    };
    chip_info
}
