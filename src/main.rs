extern crate rustc_serialize;
extern crate docopt;
extern crate tempdir;
extern crate term_painter;

use docopt::Docopt;
use std::process::Command;
use tempdir::TempDir;
use term_painter::ToStyle;
use term_painter::Color::*;

static USAGE: &'static str = "
Archive/restore a git repository to/from an AWS S3 bucket.

	Archiving: Please make sure to create the S3 bucket first.
	Restoring: Please make sure to create the empty remote git repository first.

Usage:
	gilthub archive [-p <profile>] <git-clone-url> <bucket-url>
	gilthub restore [-p <profile>] <bucket-archive-url> <git-repo-url>

Options:
	-p, --profile PROFILE     The AWS profile to use, if you have multiple profiles, you can use this option to specify the named profile to use.

Example:
	gilthub archive git@github.com:gilt/scala-1-day.git s3://github-repo-archive
	gilthub restore s3://github-repo-archive/scala-1-day.tar.gz git@github.com:grahamar/scala-1-day.git
";

#[derive(RustcDecodable)]
struct Args {
	cmd_archive: bool,
	cmd_restore: bool,
	flag_profile: String,
	arg_git_clone_url: String,
	arg_bucket_url: String,
	arg_bucket_archive_url: String,
	arg_git_repo_url: String,
}

fn main() {
	let args: Args = Docopt::new(USAGE).
		and_then(|d| d.decode()).
		unwrap_or_else(|e| e.exit());

	if args.cmd_archive {
		match run_archive(&args) {
			Ok(repo_name) => Yellow.with(|| {println!("Successfully archived {}!", repo_name)}),
			Err(e) => Red.with(|| {println!("Error archiving git repository: {}", e)}),
		}
	} else if args.cmd_restore {
		match run_restore(&args) {
			Ok(repo_name) => Yellow.with(|| {println!("Successfully restored {}!", repo_name)}),
			Err(e) => Red.with(|| {println!("Error restoring git repository: {}", e)}),
		}
	}
}

fn run_archive(args: &Args) -> Result<String, &'static str> {
	let clone_dir = match TempDir::new("gilthub") {
		Ok(dir) => dir,
		Err(e) => panic!("failed to create temp dir: {}", e),
	};

	let compress_dir = match TempDir::new("gilthub") {
		Ok(dir) => dir,
		Err(e) => panic!("failed to create temp dir: {}", e),
	};

	Green.with(|| {println!("Cloning {} to {}", Blue.paint(&args.arg_git_clone_url), Blue.paint(clone_dir.path().to_str().unwrap_or("")))});

	let clone_status = Command::new("git").arg("clone").arg("--bare").arg(&args.arg_git_clone_url).arg(clone_dir.path().to_str().unwrap_or(".")).status().unwrap_or_else(|e| {
		panic!("failed to clone: {}", e)
	});
	println!("");

	let repo_name_out = Command::new("basename").arg(&args.arg_git_clone_url).output().unwrap_or_else(|e| {
		panic!("failed to clone: {}", e)
	});

	let repo_name = String::from_utf8_lossy(&repo_name_out.stdout).replace(".git", "").trim().to_string();

	if clone_status.success() {

		let compress_status = Command::new("tar").arg("-zcf").arg(format!("{}/{}.tar.gz", compress_dir.path().to_str().unwrap_or("."), repo_name)).current_dir(clone_dir.path()).arg(".").status().unwrap_or_else(|e| {
			panic!("failed to compress: {}", e)
		});
		println!("");

		if compress_status.success() {
			Blue.with(|| {println!("Uploading {}.tar.gz to S3", repo_name)});

			let profile = match args.flag_profile.as_ref() {
				"" => "default".to_string(),
				prof => prof.to_string(),
			};

			let upload_status = Command::new("aws").arg("s3").arg("cp").arg(format!("{}/{}.tar.gz", compress_dir.path().to_str().unwrap_or("."), repo_name)).arg(format!("s3://{}", &args.arg_bucket_url)).arg("--profile").arg(profile).status().unwrap_or_else(|e| {
				panic!("failed to copy archive to s3: {}", e)
			});

			if upload_status.success() {
				Ok(repo_name)
			} else {
				Err("Unable to upload archived repository to S3.")
			}
		} else {
			Err("Unable to compress repository.")
		}
	} else {
		Err("Unable to clone repository.")
	}
}

fn run_restore(args: &Args) -> Result<String, &'static str> {
	let extract_dir = match TempDir::new("gilthub") {
		Ok(dir) => dir,
		Err(e) => panic!("failed to create temp dir: {}", e),
	};

	Green.with(|| {println!("Downloading {} to {}", Blue.paint(&args.arg_bucket_archive_url), Blue.paint(extract_dir.path().to_str().unwrap_or("")))});

	let profile = match args.flag_profile.as_ref() {
		"" => "default".to_string(),
		prof => prof.to_string(),
	};

	let download_status = Command::new("aws").arg("s3").arg("cp").arg(&args.arg_bucket_archive_url).arg(extract_dir.path().to_str().unwrap_or(".")).arg("--profile").arg(profile).status().unwrap_or_else(|e| {
		panic!("failed to download: {}", e)
	});
	println!("");

	let repo_name_out = Command::new("basename").arg(&args.arg_bucket_archive_url).output().unwrap_or_else(|e| {
		panic!("failed to clone: {}", e)
	});
	let repo_name = String::from_utf8_lossy(&repo_name_out.stdout).trim().to_string();

	if download_status.success() {

		let uncompress_status = Command::new("tar").arg("-zxf").arg(format!("{}/{}", extract_dir.path().to_str().unwrap_or("."), repo_name)).current_dir(extract_dir.path()).status().unwrap_or_else(|e| {
			panic!("failed to un-compress: {}", e)
		});
		println!("");

		if uncompress_status.success() {
			Blue.with(|| {println!("Restoring {} to {}", repo_name, &args.arg_git_repo_url)});

			let restore_status = Command::new("git").arg("push").arg("--mirror").arg(&args.arg_git_repo_url).current_dir(extract_dir.path()).status().unwrap_or_else(|e| {
				panic!("failed to restore repository: {}", e)
			});

			if restore_status.success() {
				Ok(repo_name)
			} else {
				Err("Unable to restore repository.")
			}
		} else {
			Err("Unable to un-compress repository.")
		}
	} else {
		Err("Unable to download archived repository.")
	}
}
