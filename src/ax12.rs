extern crate byteorder;

use std::fmt;
use std::io::Cursor;

use self::Access::*;
use self::Size::*;
use self::Register::*;
use self::byteorder::{LittleEndian, ReadBytesExt};

use super::packets::{Request, Status};


#[derive(Debug)]
pub enum Access {
    R,
    RW
}

#[derive(Debug)]
pub enum Size {
    Byte,
    HalfWord
}

impl Size {
    pub fn len(&self) -> usize {
        match *self {
            Size::Byte => 1,
            Size::HalfWord => 2
        }
    }
}


#[derive(Debug)]
pub struct RegisterInfo {
    pub description: &'static str,
    pub access: Access,
    pub address: u8,
    pub size: Size,
}

#[derive(Debug, Copy, Clone)]
pub enum Register {
    ModelNumber,
    FirmwareVersion,
    Id,
    BaudRate,
    ReturnDelayTime,
    CWAngleLimit,
    CCWAngleLimit,
    TemperatureLimit,
    VoltageLimitLow,
    VoltageLimitHigh,
    MaxLoad,
    StatusReturnLevel,
    AlarmLed,
    AlarmShutdown,
    TorqueEnable,
    LedEnable,
    CwComplianceMargin,
    CcwComplianceMargin,
    CwComplianceSlope,
    CcwComplianceSlope,
    GoalPosition,
    MovingSpeed,
    TorqueLimit,
    PresentPosition,
    PresentSpeed,
    PresentLoad,
    PresentVoltage,
    PresentTemperature,
    Registered,
    Moving,
    Lock,
    Punch,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub static ALL_REGISTERS: &'static [Register] = &[
    ModelNumber, FirmwareVersion,
    Id,
    BaudRate, ReturnDelayTime,
    CWAngleLimit, CCWAngleLimit,
    TemperatureLimit,
    VoltageLimitLow, VoltageLimitHigh,
    MaxLoad,
    StatusReturnLevel,
    AlarmLed, AlarmShutdown,
    TorqueEnable, LedEnable,
    CwComplianceMargin, CcwComplianceMargin,
    CwComplianceSlope, CcwComplianceSlope,
    GoalPosition,
    MovingSpeed,
    TorqueLimit,
    PresentPosition,
    PresentSpeed,
    PresentVoltage,
    PresentTemperature,
];
static REGISTER_INFO: &'static [RegisterInfo] = &[
    RegisterInfo {
        address: 0x00,
        size: HalfWord,
        access: R,
        description: "Model number"
    },
    RegisterInfo {
        address: 0x02,
        size: Byte,
        access: R,
        description: "Firmware version"
    },
    RegisterInfo {
        address: 0x03,
        size: Byte,
        access: RW,
        description: "Actuator identifier"
    },
    RegisterInfo {
        address: 0x04,
        size: Byte,
        access: RW,
        description: "Communication baud rate"
    },
    RegisterInfo {
        address: 0x05,
        size: Byte,
        access: RW,
        description: "Return delay time"
    },
    RegisterInfo {
        address: 0x06,
        size: HalfWord,
        access: RW,
        description: "Clockwise angle limit"
    },
    RegisterInfo {
        address: 0x08,
        size: HalfWord,
        access: RW,
        description: "Counterclockwise angle limit"
    },
    RegisterInfo {
        address: 0x0b,
        size: Byte,
        access: RW,
        description: "Temperature alarm level"
    },
    RegisterInfo {
        address: 0x0c,
        size: Byte,
        access: RW,
        description: "Low voltage alarm level"
    },
    RegisterInfo {
        address: 0x0d,
        size: Byte,
        access: RW,
        description: "High voltage alarm level"
    },
    RegisterInfo {
        address: 0x0e,
        size: HalfWord,
        access: RW,
        description: "Max load alarm level"
    },
    RegisterInfo {
        address: 0x10,
        size: Byte,
        access: RW,
        description: "Status return level"
    },
    RegisterInfo {
        address: 0x11,
        size: Byte,
        access: RW,
        description: "LED indication on alarm"
    },
    RegisterInfo {
        address: 0x12,
        size: Byte,
        access: RW,
        description: "Shutdown on alarm"
    },
    RegisterInfo {
        address: 0x18,
        size: Byte,
        access: RW,
        description: "Enable torque output"
    },
    RegisterInfo {
        address: 0x19,
        size: Byte,
        access: RW,
        description: "Enable Led"
    },
    RegisterInfo {
        address: 0x1a,
        size: Byte,
        access: RW,
        description: "Clockwise compliance margin"
    },
    RegisterInfo {
        address: 0x1b,
        size: Byte,
        access: RW,
        description: "Counterclockwise compliance margin"
    },
    RegisterInfo {
        address: 0x1c,
        size: Byte,
        access: RW,
        description: "Clockwise compliance slope"
    },
    RegisterInfo {
        address: 0x1d,
        size: Byte,
        access: RW,
        description: "Counterclockwise compliance slope"
    },
    RegisterInfo {
        address: 0x1e,
        size: HalfWord,
        access: RW,
        description: "Goal position"
    },
    RegisterInfo {
        address: 0x20,
        size: HalfWord,
        access: RW,
        description: "Moving speed"
    },
    RegisterInfo {
        address: 0x22,
        size: HalfWord,
        access: RW,
        description: "Torque limit"
    },
    RegisterInfo {
        address: 0x24,
        size: HalfWord,
        access: R,
        description: "Current position"
    },
    RegisterInfo {
        address: 0x26,
        size: HalfWord,
        access: R,
        description: "Current speed"
    },
    RegisterInfo {
        address: 0x28,
        size: HalfWord,
        access: R,
        description: "Current load"
    },
    RegisterInfo {
        address: 0x2a,
        size: Byte,
        access: R,
        description: "Current voltage"
    },
    RegisterInfo {
        address: 0x2b,
        size: Byte,
        access: R,
        description: "Current temperature"
    },
    RegisterInfo {
        address: 0x2c,
        size: Byte,
        access: R,
        description: "Instruction registered"
    },
    RegisterInfo {
        address: 0x2e,
        size: Byte,
        access: R,
        description: "Is Moving"
    },
    RegisterInfo {
        address: 0x2f,
        size: Byte,
        access: RW,
        description: "EEPROM Lock"
    },
    RegisterInfo {
        address: 0x30,
        size: HalfWord,
        access: RW,
        description: "Punch value"
    },
];

impl Register {
    pub fn info(&self) -> &'static RegisterInfo {
        &REGISTER_INFO[*self as usize]
    }

    pub fn read_request(&self, id: u8) -> Request {
        let info = self.info();
        Request::Read { id: id, addr: info.address, len: info.size.len() as u8 }
    }

    pub fn parse_read_value(&self, status: Status) -> u16 {
        if status.data.len() == 1 {
            status.data[0] as u16
        } else {
            let mut rdr = Cursor::new(status.data);
            rdr.read_u16::<LittleEndian>().unwrap()
        }
    }
}