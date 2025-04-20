use anyhow::{Result, anyhow};
use std::io;
use win_ctx::{ActivationType, CtxEntry, EntryOptions};

pub struct CtxMenu<'a> {
	pub name: &'a str,
	pub entry_type: ActivationType,
	pub opts: Option<EntryOptions>,
}

pub fn delete_ctx_menu(menu: &CtxMenu) -> Result<()> {
	let ctx = CtxEntry::get(&[menu.name], &menu.entry_type)
		.ok_or(anyhow!("No context menu by that path"))?;
	println!("Deleted Key: {}", ctx.path());
	ctx.delete()?;
	Ok(())
}
pub fn create_ctx_menu(menu: &CtxMenu) -> Result<CtxEntry> {
	let ctx = match &menu.opts {
		Some(opts) => {
			CtxEntry::new_with_options(menu.name, &menu.entry_type, opts)
		}
		None => CtxEntry::new(menu.name, &menu.entry_type),
	};

	match ctx {
		Ok(ctx) => {
			println!("Created Key: {}", ctx.path());
			Ok(ctx)
		}
		Err(err) => match err.kind() {
			io::ErrorKind::AlreadyExists => {
				let ctx = CtxEntry::get(&[menu.name], &menu.entry_type)
					.ok_or(anyhow!("No context menu by that path"))?;
				println!("Created Key: {}", ctx.path());
				ctx.delete()?;
				Ok(match &menu.opts {
					Some(opts) => CtxEntry::new_with_options(
						menu.name,
						&menu.entry_type,
						opts,
					),
					None => CtxEntry::new(menu.name, &menu.entry_type),
				}?)
			}
			_ => Err(anyhow!("Failed to create context menu")),
		},
	}
}
