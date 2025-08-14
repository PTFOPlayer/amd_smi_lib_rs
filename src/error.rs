use amd_smi_lib_sys::bindings::amdsmi_status_t;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum AmdSmiError {
    AmdsmiStatusInval = 1,
    AmdsmiStatusNotSupported = 2,
    AmdsmiStatusNotYetImplemented = 3,
    AmdsmiStatusFailLoadModule = 4,
    AmdsmiStatusFailLoadSymbol = 5,
    AmdsmiStatusDrmError = 6,
    AmdsmiStatusApiFailed = 7,
    AmdsmiStatusTimeout = 8,
    AmdsmiStatusRetry = 9,
    AmdsmiStatusNoPerm = 10,
    AmdsmiStatusInterrupt = 11,
    AmdsmiStatusIo = 12,
    AmdsmiStatusAddressFault = 13,
    AmdsmiStatusFileError = 14,
    AmdsmiStatusOutOfResources = 15,
    AmdsmiStatusInternalException = 16,
    AmdsmiStatusInputOutOfBounds = 17,
    AmdsmiStatusInitError = 18,
    AmdsmiStatusRefcountOverflow = 19,
    AmdsmiStatusBusy = 30,
    AmdsmiStatusNotFound = 31,
    AmdsmiStatusNotInit = 32,
    AmdsmiStatusNoSlot = 33,
    AmdsmiStatusDriverNotLoaded = 34,
    AmdsmiStatusNoData = 40,
    AmdsmiStatusInsufficientSize = 41,
    AmdsmiStatusUnexpectedSize = 42,
    AmdsmiStatusUnexpectedData = 43,
    AmdsmiStatusNonAmdCpu = 44,
    AmdsmiStatusNoEnergyDrv = 45,
    AmdsmiStatusNoMsrDrv = 46,
    AmdsmiStatusNoHsmpDrv = 47,
    AmdsmiStatusNoHsmpSup = 48,
    AmdsmiStatusNoHsmpMsgSup = 49,
    AmdsmiStatusHsmpTimeout = 50,
    AmdsmiStatusNoDrv = 51,
    AmdsmiStatusFileNotFound = 52,
    AmdsmiStatusArgPtrNull = 53,
    AmdsmiStatusAmdgpuRestartErr = 54,
    AmdsmiStatusSettingUnavailable = 55,
    AmdsmiStatusCorruptedEeprom = 56,
    AmdsmiStatusMapError = 4294967294,
    AmdsmiStatusUnknownError = 4294967295,
}

fn result_from_amd_smi_status_t(value: amdsmi_status_t) -> Result<(), AmdSmiError> {
    if value != 0 {
        unsafe {
            return Err(std::mem::transmute(value));
        }
    }

    Ok(())
}

pub trait IntoAmdSmiResult {
    fn into_amd_smi_result(self) -> Result<(), AmdSmiError>;
}

impl IntoAmdSmiResult for amdsmi_status_t {
    fn into_amd_smi_result(self) -> Result<(), AmdSmiError> {
        result_from_amd_smi_status_t(self)
    }
}
