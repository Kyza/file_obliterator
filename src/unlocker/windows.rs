use core::str;
use std::{
	ffi::c_void,
	process::Command,
	thread::{self, JoinHandle},
};

use anyhow::{Result, anyhow};
use windows::{
	Wdk::{
		Storage::FileSystem::{
			FileProcessIdsUsingFileInformation, NtQueryInformationFile,
		},
		System::SystemServices::FILE_PROCESS_IDS_USING_FILE_INFORMATION,
	},
	Win32::{
		Foundation::{CloseHandle, STATUS_SUCCESS},
		Storage::FileSystem::{
			CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ,
			OPEN_EXISTING,
		},
		System::{
			IO::IO_STATUS_BLOCK,
			Threading::{
				INFINITE, OpenProcess, PROCESS_SYNCHRONIZE,
				PROCESS_TERMINATE, TerminateProcess, WaitForSingleObject,
			},
		},
	},
	core::{HSTRING, PCWSTR},
};

pub fn clean_restart_explorer() -> Result<()> {
	let output = Command::new("powershell")
		.args([
			"-Command",
			"((New-Object -com shell.application).Windows() | ForEach-Object { $_.Document.Folder.Self.Path }) -join \"`n\""
		])
		.output()?;

	let paths_str = str::from_utf8(&output.stdout)?.trim();

	let open_folders = paths_str
		.lines()
		.map(|line| line.trim().to_string())
		.filter(|s| !s.is_empty())
		.collect::<Vec<String>>();

	Command::new("taskkill")
		.arg("/IM")
		.arg("explorer.exe")
		.arg("/F")
		.output()?;

	Command::new("explorer").output()?;

	for folder in open_folders {
		Command::new("explorer").arg(folder).output()?;
	}

	Ok(())
}

fn to_pcwstr_owned(s: impl ToString) -> (HSTRING, PCWSTR) {
	let hstring = HSTRING::from(s.to_string());
	let pcwstr = PCWSTR::from_raw(hstring.as_ptr());
	(hstring, pcwstr)
}

/// Loops through files in parallel and attempts to kill any processes locking them.
/// Returns true if any processes were killed, false otherwise.
/// Returns an error if anything goes wrong.
pub unsafe fn unlock_files(file_paths: Vec<impl ToString>) -> Result<bool> {
	let handles: Vec<JoinHandle<Result<bool>>> = file_paths
		.iter()
		.map(|file_path| file_path.to_string())
		.map(|file_path| {
			thread::spawn(move || {
				let file_handle = unsafe {
					CreateFileW(
						to_pcwstr_owned(file_path.to_string()).1,
						0,
						FILE_SHARE_READ,
						None,
						OPEN_EXISTING,
						FILE_FLAG_BACKUP_SEMANTICS,
						None,
					)?
				};

				let mut io_status_block = IO_STATUS_BLOCK::default();
				let mut file_information =
					FILE_PROCESS_IDS_USING_FILE_INFORMATION::default();
				let status = unsafe {
					NtQueryInformationFile(
						file_handle,
						&mut io_status_block as *mut _,
						&mut file_information as *mut _ as *mut c_void,
						std::mem::size_of::<
							FILE_PROCESS_IDS_USING_FILE_INFORMATION,
						>() as u32,
						FileProcessIdsUsingFileInformation,
					)
				};
				if status == STATUS_SUCCESS {
					if file_information.NumberOfProcessIdsInList == 0 {
						return Ok(false);
					}
					let mut process_handles = Vec::new();
					for process_id in file_information.ProcessIdList {
						println!("Killing process: {:?}", file_information);
						let process_handle = unsafe {
							OpenProcess(
								PROCESS_TERMINATE | PROCESS_SYNCHRONIZE,
								false,
								process_id as u32,
							)?
						};
						unsafe { TerminateProcess(process_handle, 1) }?;
						process_handles.push(process_handle);
					}
					for process_handle in process_handles {
						// Waits for the process to close.
						unsafe {
							WaitForSingleObject(process_handle, INFINITE)
						};
						unsafe { CloseHandle(process_handle) }?;
					}
				} else {
					unsafe { CloseHandle(file_handle) }?;
					return Err(anyhow!("Failed to query file information"));
				}

				unsafe { CloseHandle(file_handle) }?;

				Ok(true)
			})
		})
		.collect();

	let mut result = false;
	for handle in handles {
		result |= handle.join().map_err(|_| anyhow!("Thread panicked"))??;
	}

	Ok(result)
}
