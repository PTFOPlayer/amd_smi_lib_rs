use amd_smi_lib_sys::bindings::{
    self, amdsmi_init_flags_t_AMDSMI_INIT_AMD_APUS, amdsmi_init_flags_t_AMDSMI_INIT_AMD_CPUS,
    amdsmi_init_flags_t_AMDSMI_INIT_AMD_GPUS, amdsmi_init_flags_t_AMDSMI_INIT_NON_AMD_CPUS,
    amdsmi_init_flags_t_AMDSMI_INIT_NON_AMD_GPUS,
};
use bitflags::bitflags;

use crate::error::{AmdSmiError, IntoAmdSmiResult};
pub mod discovery;
pub mod error;
pub mod handles;
pub mod util;

bitflags! {
   pub struct InitFlags: u32 {
        const CPU = amdsmi_init_flags_t_AMDSMI_INIT_AMD_CPUS;
        const GPU = amdsmi_init_flags_t_AMDSMI_INIT_AMD_GPUS;
        const APU = amdsmi_init_flags_t_AMDSMI_INIT_AMD_APUS;
        const NON_AMD_CPU = amdsmi_init_flags_t_AMDSMI_INIT_NON_AMD_CPUS;
        const NON_AMD_GPU = amdsmi_init_flags_t_AMDSMI_INIT_NON_AMD_GPUS;
    }
}

impl InitFlags {
    pub fn as_u64(&self) -> u64 {
        self.bits() as u64
    }
}

pub struct AmdSmi {
    mode: InitFlags,
}

impl AmdSmi {
    /// Currently only woking flag is GPU flag
    ///
    /// # Errors
    ///
    /// This function will return an error if flags are zero or unsupported
    pub unsafe fn with_flags(flags: InitFlags) -> Result<Self, AmdSmiError> {
        unsafe { bindings::amdsmi_init(flags.as_u64()) }.into_amd_smi_result()?;

        Ok(Self { mode: flags })
    }

    /// Initializes AMDSMI only for gpu functions.
    ///
    /// # Errors
    ///
    /// This function will return an error if initialization is not possible.
    pub fn init_gpu() -> Result<Self, AmdSmiError> {
        unsafe { Self::with_flags(InitFlags::GPU) }
    }
}

impl Drop for AmdSmi {
    fn drop(&mut self) {
        unsafe { bindings::amdsmi_shut_down() }
            .into_amd_smi_result()
            .map_err(|err| panic!("Panic while cleaning up: {:?}", err))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::AmdSmi;

    #[test]
    pub fn main_test() {
        AmdSmi::init_gpu().unwrap();
    }
}
