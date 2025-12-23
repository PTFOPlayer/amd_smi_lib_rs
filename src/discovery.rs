use crate::util::StringCleanup;
use std::ptr::null_mut;

use amd_smi_lib_sys::bindings::{
    AMDSMI_GPU_UUID_SIZE, AMDSMI_MAX_STRING_LENGTH, amdsmi_bdf_t, amdsmi_get_gpu_device_bdf, amdsmi_get_gpu_device_uuid, amdsmi_get_gpu_id, amdsmi_get_gpu_revision, amdsmi_get_gpu_subsystem_id, amdsmi_get_gpu_subsystem_name, amdsmi_get_gpu_vendor_name, amdsmi_get_processor_handles, amdsmi_get_processor_info, amdsmi_get_processor_type, amdsmi_get_socket_handles, amdsmi_get_socket_info
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
        id: u16,
        revision: u16,
        vendor_name: String,
        subsystem_id: u16,  
        subsystem_name: String,
    },
    AmdCpu {
        name: String,
    },
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

        let mut name = String::from_utf8(vec![0; 256]).unwrap();
        for socket in socket_handles {
            unsafe {
                amdsmi_get_socket_info(socket, 256, name.as_mut_ptr().cast())
                    .into_amd_smi_result()?;
            }

            let processors = Self::get_processors_info(socket)?;
            sockets.push(SocketInfo {
                name: name.clone().string_cleanup(),
                processors,
            });
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
                2 => Ok(Self::get_cpu_info(processor_handle)?),
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
        let mut id = 0u16;
        let mut revision = 0u16;
        let mut vendor_name =
            String::from_utf8(vec![0; AMDSMI_MAX_STRING_LENGTH as usize]).unwrap();
        let mut subsystem_id = 0u16;
        let mut subsystem_name =
            String::from_utf8(vec![0; AMDSMI_MAX_STRING_LENGTH as usize]).unwrap();

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

            amdsmi_get_gpu_id(processor_handle, &mut id).into_amd_smi_result()?;

            amdsmi_get_gpu_revision(processor_handle, &mut revision).into_amd_smi_result()?;

            amdsmi_get_gpu_vendor_name(
                processor_handle,
                vendor_name.as_mut_ptr().cast(),
                AMDSMI_MAX_STRING_LENGTH as usize,
            )
            .into_amd_smi_result()?;

            amdsmi_get_gpu_subsystem_id(processor_handle, &mut subsystem_id)
                .into_amd_smi_result()?;

            amdsmi_get_gpu_subsystem_name(
                processor_handle,
                subsystem_name.as_mut_ptr().cast(),
                AMDSMI_MAX_STRING_LENGTH as usize,
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
            id,
            revision,
            vendor_name: vendor_name.string_cleanup(),
            subsystem_id,
            subsystem_name: subsystem_name.string_cleanup(),
        })
    }

    fn get_cpu_info(processor_handle: ProcessorHandle) -> Result<ProcessorType, AmdSmiError> {
        let mut name = String::from_utf8(vec![0; 256]).unwrap();

        unsafe {
            amdsmi_get_processor_info(processor_handle, 256, name.as_mut_ptr().cast())
                .into_amd_smi_result()?;
        }
        name = name.string_cleanup();

        Ok(ProcessorType::AmdCpu { name })
    }
}

#[cfg(test)]
mod discovery_tests {
    use crate::{AmdSmi, error::AmdSmiError};

    #[test]
    fn test_discovery() -> Result<(), AmdSmiError> {
        // scoping to ensure amdsmi is shutdown before re-initializing

        let cpu_count = {
            let mut amdsmi = AmdSmi::init_cpu()?;
            amdsmi.get_sockets_info()?.len()
        };

        let gpu_count = {
            let mut amdsmi = AmdSmi::init_gpu()?;
            amdsmi.get_sockets_info()?.len()
        };

        let total = {
            let mut amdsmi = AmdSmi::init_all()?;
            amdsmi.get_sockets_info()?
        };

        println!("{:?}", total);

        assert_eq!(total.len(), cpu_count + gpu_count);
        Ok(())
    }
}
