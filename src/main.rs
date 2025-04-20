use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

mod ctx_menu;
mod unlocker;
use ctx_menu::{CtxMenu, create_ctx_menu, delete_ctx_menu};

use anyhow::Result;
use serde::Serialize;
use unlocker::windows::clean_restart_explorer;
use walkdir::WalkDir;
use win_ctx::{ActivationType, EntryOptions, Separator, toggle_classic_menu};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Default, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Action {
	/// Run setup.
	#[default]
	Setup,
	/// Run unsetup.
	Unsetup,
	/// Unlock files.
	Unlock,
	/// Unlock and delete files.
	Obliterate,
}
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// Action to take.
	#[arg(short, long, default_value = "setup")]
	action: Action,

	/// Files to take action on.
	#[arg(short, long)]
	files: Vec<String>,
}

static CTX_MENUS: LazyLock<Vec<CtxMenu>> = LazyLock::new(|| {
	vec![
		CtxMenu {
			name: "Unlock File",
			entry_type: ActivationType::File("*".to_string()),
			opts: Some(EntryOptions {
				command: Some(format!(
					"{} -a unlock -f \"%V\"",
					env::current_exe()
						.expect("failed to find executable path")
						.to_string_lossy()
				)),
				icon: None,
				position: None,
				separator: Some(Separator::Before),
				extended: false,
			}),
		},
		CtxMenu {
			name: "Obliterate File",
			entry_type: ActivationType::File("*".to_string()),
			opts: Some(EntryOptions {
				command: Some(format!(
					"{} -a obliterate -f \"%V\"",
					env::current_exe()
						.expect("failed to find executable path")
						.to_string_lossy()
				)),
				icon: None,
				position: None,
				separator: Some(Separator::After),
				extended: false,
			}),
		},
		CtxMenu {
			name: "Unlock Folder",
			entry_type: ActivationType::Folder,
			opts: Some(EntryOptions {
				command: Some(format!(
					"{} -a unlock -f \"%V\"",
					env::current_exe()
						.expect("failed to find executable path")
						.to_string_lossy()
				)),
				icon: None,
				position: None,
				separator: Some(Separator::Before),
				extended: false,
			}),
		},
		CtxMenu {
			name: "Obliterate Folder",
			entry_type: ActivationType::Folder,
			opts: Some(EntryOptions {
				command: Some(format!(
					"{} -a obliterate -f \"%V\"",
					env::current_exe()
						.expect("failed to find executable path")
						.to_string_lossy()
				)),
				icon: None,
				position: None,
				separator: Some(Separator::After),
				extended: false,
			}),
		},
		CtxMenu {
			name: "Unlock Folder",
			entry_type: ActivationType::Background,
			opts: Some(EntryOptions {
				command: Some(format!(
					"{} -a unlock -f \"%V\"",
					env::current_exe()
						.expect("failed to find executable path")
						.to_string_lossy()
				)),
				icon: None,
				position: None,
				separator: Some(Separator::Before),
				extended: false,
			}),
		},
		CtxMenu {
			name: "Obliterate Folder",
			entry_type: ActivationType::Background,
			opts: Some(EntryOptions {
				command: Some(format!(
					"{} -a obliterate -f \"%V\"",
					env::current_exe()
						.expect("failed to find executable path")
						.to_string_lossy()
				)),
				icon: None,
				position: None,
				separator: Some(Separator::After),
				extended: false,
			}),
		},
	]
});

fn start() -> Result<()> {
	let args = Args::parse();

	if args.action == Action::Setup {
		toggle_classic_menu(true)?;

		for menu in &*CTX_MENUS {
			create_ctx_menu(menu)?;
		}

		println!("Registered context menu items.");

		// Refreshes the context menu if the W11 one was active.
		clean_restart_explorer()?;

		println!("Clean restarted explorer.exe.");

		return Ok(());
	} else if args.action == Action::Unsetup {
		for menu in &*CTX_MENUS {
			delete_ctx_menu(menu)?;
		}

		println!("Unregistered context menu items.");

		return Ok(());
	}

	// A path here can be either a file or a folder.
	// Add folders with their contents recursively.
	let paths = args
		.files
		.clone()
		.into_iter()
		.flat_map(|path| {
			let path = PathBuf::from(path);
			if path.is_dir() {
				let mut entries = Vec::new();
				// This will yield the root folder automatically.
				for entry in WalkDir::new(path).contents_first(true) {
					entries.push(
						entry
							.expect("Failed to read directory entry")
							.into_path(),
					);
				}
				entries
			} else {
				vec![path]
			}
		})
		.collect::<Vec<_>>();

	// Print them.
	for path in paths.clone() {
		println!("{}", path.to_string_lossy());
	}

	// Filter all the folders.
	let paths_no_folders_winapi = paths
		.clone()
		.into_iter()
		.filter(|path| !path.is_dir())
		.map(|path| path.to_string_lossy().to_string())
		.collect::<Vec<_>>();

	match args.action {
		Action::Unlock => {
			unsafe {
				unlocker::windows::unlock_files(paths_no_folders_winapi)
			}?;
		}
		Action::Obliterate => {
			println!("This will delete the above files.");
			println!("Are you sure? (y/n)");
			let mut input = String::new();
			std::io::stdin().read_line(&mut input).unwrap();
			if input.trim().to_lowercase() == "y" {
				unsafe {
					unlocker::windows::unlock_files(paths_no_folders_winapi)
				}?;
				for path in paths {
					if path.exists() {
						println!("Deleting: {}", path.to_string_lossy());
						if path.is_dir() {
							// If fails to delete directory, stop explorer.exe and retry.
							if fs::remove_dir_all(path.clone()).is_err() {
								println!(
									"Failed to delete. This is likely the \
									 folder open in explorer.exe."
								);
							}
						} else {
							fs::remove_file(path)?;
						}
					}
				}
			}
		}
		_ => {}
	}
	Ok(())
}

fn main() -> Result<()> {
	if let Err(err) = start() {
		eprintln!("Error: {}", err);
		std::thread::sleep(std::time::Duration::from_secs(60));
	}
	Ok(())
}
