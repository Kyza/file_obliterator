extern crate winres;

fn main() {
	if cfg!(target_os = "windows") {
		let mut res = winres::WindowsResource::new();
		res.set_manifest_file("./Package.appxmanifest");
		res.compile().unwrap();
	} else {
		panic!(
			"This program currently only works on Windows.\nIf you'd like \
			 it on your platform please make a PR!"
		);
	}
}
