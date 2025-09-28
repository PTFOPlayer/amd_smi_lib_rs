use crate::util::StringCleanup;
use std::ptr::null_mut;

use amd_smi_lib_sys::bindings::{
    AMDSMI_GPU_UUID_SIZE, amdsmi_bdf_t, amdsmi_enumeration_info_t, amdsmi_get_gpu_device_bdf,
    amdsmi_get_gpu_device_uuid, amdsmi_get_gpu_enumeration_info,
    amdsmi_get_gpu_virtualization_mode, amdsmi_get_processor_handles, amdsmi_get_processor_type,
    amdsmi_get_socket_handles, amdsmi_get_socket_info,
};

use crate::{
    AmdSmi,
    error::{AmdSmiError, IntoAmdSmiResult},
    handles::{ProcessorHandle, SocketHandle},
};

#[derive(Debug, Clone)]
pub enum ProcessorType {
    Unknown,
    AmdGpu {
        bdf: BDF,
        uuid: String,
        drm_render: u32,
        drm_card: u32,
        hsa_id: u32,
        hip_id: u32,
        hip_uuid: String,
        virtualization_mode: VirtualizationMode
    },
    AmdCpu,
    NonAmdGpu,
    NonAmdCpu,
    AmdCpuCore,
    AmdApu,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum VirtualizationMode {
    Unknown,
    BareMetal,
    Host,
    Guest,
    PassThrough,
}

#[derive(Debug, Clone, Copy)]
pub struct BDF {
    pub function_number: u64,
    pub device_number: u64,
    pub bus_number: u64,
    pub domain_number: u64,
}

#[derive(Debug, Clone)]
pub struct SocketInfo {
    pub name: String,
    pub processors: Vec<ProcessorType>,
}

impl AmdSmi {
    pub fn get_sockets_info(&mut self) -> Result<Vec<SocketInfo>, AmdSmiError> {
        let mut socket_count = 0u32;
        unsafe {
            amdsmi_get_socket_handles(&mut socket_count as *mut u32, null_mut())
                .into_amd_smi_result()?;
        }

        let mut socket_handles = vec![SocketHandle::default(); socket_count as usize];
        unsafe {
            amdsmi_get_socket_handles(&mut socket_count, socket_handles.as_mut_ptr())
                .into_amd_smi_result()?;
        }

        let mut sockets = vec![];

        let mut raw_name = vec![0i8; 256];
        for socket in socket_handles {
            unsafe {
                amdsmi_get_socket_info(socket, 256, raw_name.as_mut_ptr()).into_amd_smi_result()?;
            }

            let name = String::from_utf8(raw_name.iter().map(|i| i.cast_unsigned()).collect())
                .map_err(|_| AmdSmiError::AmdsmiStatusUnexpectedData)?
                .string_cleanup();

            let processors = Self::get_processors_info(socket)?;
            sockets.push(SocketInfo { name, processors });
        }

        Ok(sockets)
    }

    fn get_processors_info(socket_handle: SocketHandle) -> Result<Vec<ProcessorType>, AmdSmiError> {
        let mut processor_count = 0u32;
        unsafe {
            amdsmi_get_processor_handles(socket_handle, &mut processor_count, null_mut())
                .into_amd_smi_result()?;
        }

        let mut processor_handles = vec![ProcessorHandle::default(); processor_count as usize];
        unsafe {
            amdsmi_get_processor_handles(
                socket_handle,
                &mut processor_count,
                processor_handles.as_mut_ptr(),
            )
            .into_amd_smi_result()?;
        }

        let mut processors_info = vec![];

        for processor_handle in processor_handles {
            let mut processor_type = 0u32;
            unsafe {
                amdsmi_get_processor_type(processor_handle, &mut processor_type);
            }

            let processor_type = match processor_type {
                0 => Ok(ProcessorType::Unknown),
                1 => Ok(Self::get_gpu_info(processor_handle)?),
                2 => Ok(ProcessorType::AmdCpu),
                3 => Ok(ProcessorType::NonAmdGpu),
                4 => Ok(ProcessorType::NonAmdCpu),
                5 => Ok(ProcessorType::AmdCpuCore),
                6 => Ok(ProcessorType::AmdApu),
                _ => Err(AmdSmiError::AmdsmiStatusUnexpectedData),
            }?;

            processors_info.push(processor_type);
        }

        Ok(processors_info)
    }

    fn get_gpu_info(processor_handle: ProcessorHandle) -> Result<ProcessorType, AmdSmiError> {
        let mut bdf = 0u64;

        let mut uuid_length = AMDSMI_GPU_UUID_SIZE;
        let mut uuid = String::from_utf8(vec![0; AMDSMI_GPU_UUID_SIZE as usize]).unwrap();

        let mut enumeration_info: amdsmi_enumeration_info_t = unsafe { std::mem::zeroed() };

        let mut virtualization_mode: VirtualizationMode = VirtualizationMode::Unknown;

        unsafe {
            amdsmi_get_gpu_device_bdf(
                processor_handle,
                (&mut bdf as *mut u64).cast::<amdsmi_bdf_t>(),
            )
            .into_amd_smi_result()?;

            amdsmi_get_gpu_device_uuid(
                processor_handle,
                &mut uuid_length,
                uuid.as_mut_ptr().cast(),
            )
            .into_amd_smi_result()?;

            amdsmi_get_gpu_enumeration_info(processor_handle, &mut enumeration_info)
                .into_amd_smi_result()?;

            amdsmi_get_gpu_virtualization_mode(
                processor_handle,
                (&mut virtualization_mode as *mut VirtualizationMode).cast(),
            )
            .into_amd_smi_result()?;
        }

        Ok(ProcessorType::AmdGpu {
            bdf: BDF {
                function_number: (bdf >> 0) & 0b111,           // 3 bits
                device_number: (bdf >> 3) & 0b1_1111,          // 5 bits
                bus_number: (bdf >> 8) & 0b1111_1111,          // 8 bits
                domain_number: (bdf >> 16) & 0xFFFF_FFFF_FFFF, // 48 bits
            },
            uuid: uuid.string_cleanup(),
            drm_render: enumeration_info.drm_render,
            drm_card: enumeration_info.drm_card,
            hsa_id: enumeration_info.hsa_id,
            hip_id: enumeration_info.hip_id,
            hip_uuid: String::from_utf8_lossy(
                &enumeration_info
                    .hip_uuid
                    .iter()
                    .map(|i| i.cast_unsigned())
                    .collect::<Vec<_>>(),
            )
            .string_cleanup(),
            virtualization_mode,
        })
    }
}

#[cfg(test)]
mod discovery_tests {
    use crate::{AmdSmi, error::AmdSmiError};

    #[test]
    fn test_socket_info() -> Result<(), AmdSmiError> {
        let mut amdsmi = AmdSmi::init_gpu()?;

        println!("{:?}", amdsmi.get_sockets_info()?);
        Ok(())
    }
}
