#[macro_use]
extern crate clap;
use std::borrow::BorrowMut;
use std::cmp::min;
use clap::App;
use walkdir::WalkDir;
use std::path::{PathBuf, Path};
use std::process::Command;

use threadpool::ThreadPool;
use std::sync::{Arc, Mutex};

// TODO: Convert all string numbers to actual numbers
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct VideoTrack {
	codec: String,
	bitrate: String,
	height: String,
	scantype:  String,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AudioTrack {
	codec: String,
	bitrate: String,
	bit_mode: String,	// CBR vs VBR
	channels: String,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SubTrack {
	language: String,
	codec: String,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MediaInfo {
	filename: String,
	container: String,
	video_tracks: Vec<VideoTrack>,
	audio_tracks: Vec<AudioTrack>,
	sub_tracks: Vec<SubTrack>,
}

// There's probably a more optimal way to do this
fn format_bitrate(input: String) -> String {
	if let Ok(mut value) = input.parse::<f64>() {
		let suffix = match input.len() {
			4..=6 => {value /= 1000f64; "kb/s"},
			7..=9 => {value /= 1000000f64; "mb/s"},
			_ => "bps",
		};

		let mut value = format!("{:.3}", value);
		value.truncate(5);

		format!("{0} {1}", value, suffix)
	}
	else {
		String::from("")
	}
}

impl VideoTrack {
	fn new(blob: &json::JsonValue) -> VideoTrack {
		VideoTrack {
			codec: String::from(match blob["CodecID"].as_str().unwrap_or("") {
				"V_MPEGH/ISO/HEVC" => "HEVC",
				"V_MPEG4/ISO/AVC" => "h264", // Better alignment/at-a-glance
				"avc1" => "h264",
				_ => blob["CodecID"].as_str().unwrap()
			}),
			bitrate: format_bitrate(blob["BitRate"].to_string()),
			height: blob["Height"].to_string(),
			scantype: String::from(match blob["ScanType"].as_str().unwrap_or("") {
				"Progressive" => "p",
				"Interlaced" => "i",
				_ => ""
			}),
		}
	}
}

impl AudioTrack {
	fn new(blob: &json::JsonValue) -> AudioTrack {
		AudioTrack {
			codec: {
				if let Some(cn) = blob["Format_Commercial_IfAny"].as_str() {
					// There has got to be a better way to get these...
					String::from(match cn {
						"DTS-HD Master Audio" => "DTSHD-MA",
						"Dolby Digital Plus with Dolby Atmos" => "DDP + Atmos",
						"Dolby TrueHD with Dolby Atmos" => "TrueHD + Atmos",
						_ => cn
					})
				}
				else {
					let ret = blob["CodecID"].as_str().unwrap();
					match ret {
						"55" => String::from("MP3"),
						_ => {
							let mut ret = String::from(ret);
							if ret.starts_with("A_") {
								ret.drain(0..2);
							};
							ret
						}
					}
				}
			},
			bitrate: format_bitrate(blob["BitRate"].to_string()),
			bit_mode: blob["BitRate_Mode"].to_string(),
			channels: String::from(match blob["Channels"].to_string().parse::<i32>().unwrap() {
				8 => "7.1",
				6 => "5.1",
				2 => "Stereo",
				1 => "Mono",
				_ => blob["Channels"].as_str().unwrap()
			})
		}
	}
}

impl SubTrack {
	fn new(blob: &json::JsonValue) -> SubTrack {
		SubTrack {
			codec: blob["Format"].to_string(),
			language: blob["Language"].to_string(),
		}
	}
}


impl MediaInfo {
	fn parse(filename: String, md: String) -> Option<MediaInfo> {
		let js = json::parse(md.as_str()).unwrap();

		let mut container: Option<String> = None;
		let mut video = Vec::<VideoTrack>::new();
		let mut audio = Vec::<AudioTrack>::new();
		let mut sub = Vec::<SubTrack>::new();

		let tracks = js["media"]["track"].members();

		for t in tracks {
			match t["@type"].to_string().as_str() {
				"General" => container = Some(t["Format"].to_string()),
				"Video" => video.push(VideoTrack::new(t)),
				"Audio" => audio.push(AudioTrack::new(t)),
				"Text" => sub.push(SubTrack::new(t)),
				_ => (),
			};
		}

		// Bail early, no container
		if container.is_none() {
			return None
		}

		let ret = MediaInfo {
			filename: filename,
			container: container.unwrap(),
			video_tracks: video,
			audio_tracks: audio,
			sub_tracks: sub,
		};

		Some(ret)
	}

	fn get_video_printline(&self, vts: usize) -> Vec<String> {
		let mut ret = Vec::new();
		let mut vts = vts;

		for l in self.video_tracks.iter() {
			if vts == 0 {
				break;
			}
			ret.push(l.codec.clone());
			ret.push(l.bitrate.clone());
			ret.push(format!("{}{}", l.height, l.scantype));
			vts -= 1;
		}
		for _ in 0..vts {
			ret.push(String::new());
		}

		ret
	}

	fn get_audio_printline(&self, ats: usize) -> Vec<String> {
		let mut ret = Vec::new();
		let mut ats = ats;

		for l in self.audio_tracks.iter() {
			if ats == 0 {
				break;
			}
			ret.push(l.codec.clone());
			ret.push(l.bitrate.clone());
			ret.push(l.bit_mode.clone());
			ret.push(l.channels.clone());
			ats -= 1;
		}
		for _ in 0..ats {
			ret.push(String::new());
		}

		ret
	}


	// TODO: Custom format certain outputs
	fn get_printline(&self, vts: usize, ats: usize, _sts: usize) -> Vec<String> {
		let mut ret = Vec::new();
		let mut path = self.filename.clone(); // TODO: probably can just remove this

		// TODO: parameterize this? put this in the print logic?
		let mpl = 42;

		// TODO: de-complicate this
		if path.len() > mpl {
			let diff = (path.len() - mpl) / 2 + 1; // Get how much we are over by
			let dm = path.len() % 2;               // Get integer divide offset
			path.drain(path.len()/2 - diff ..= path.len()/2 + diff - dm);
			path.insert(path.len()/2 - dm, '…');
		}

		ret.push(path);
		ret.push(String::from("|"));
		ret.extend(self.get_video_printline(vts));
		ret.push(String::from("|"));
		ret.extend(self.get_audio_printline(ats));
		//ret.extend(self.get_sub_printline(sts));
/*
		for i in 0..sts {
			ret.push(&self.sub_tracks[i].codec);
			ret.push(&self.sub_tracks[i].language);
		}
*/
		ret
	}

}


// Should almost definitely be a result
fn get_mediainfo_output(path: PathBuf) -> String {
	let pathstr = path.into_os_string();
	// TODO: check for mediainfo in path
	let cmd = Command::new("mediainfo")
				.arg("--Output=JSON")
				.arg(&pathstr)
				.output()
				.expect("mediainfo failed to open file");


	std::str::from_utf8(&cmd.stdout).unwrap().to_owned()
}


// TODO: Optimize?
fn filter_by_extension(file: &walkdir::DirEntry) -> bool {
	let ext = file.path().extension();

	let ext = match ext {
		Some(t) => t,
		_ => return false
	};

	match ext.to_str().unwrap() {
		"mkv"|"avi"|"mpg"|"mp4" => true,
		_ => false
	}
}

// Get all the files we want to spam mediainfo on
fn get_media_paths(path: &Path, depth: Option<usize>) -> Vec<PathBuf> {
	// TODO: Parameterize the recursive depth
	let mut ret = Vec::<PathBuf>::new();
	let depth = depth.unwrap_or(1);

	// Saving these in case... I have a feeling performance may be an issue
	//.filter_entry(|x| filter_by_extension(x)) {
	//.filter_entry(|x| x.file_type().is_file()){
	// 	&& filter_by_extension(x)) {

	for entry in WalkDir::new(path)
						.min_depth(1)
						.max_depth(depth)
						.into_iter()
						// Keep dirs, we may need to recurse. Otherwise filter out non-matching files
						.filter_entry(|x| x.file_type().is_dir()
							|| filter_by_extension(x)) {
	 	let entry = match entry {
			Ok(e) => e,
			Err(e) => { println!("Error: {:?}", e); continue },
		};

		// Ignore dir, kept so walk can recurse
		if entry.file_type().is_dir() {
			continue;
		}

		ret.push(entry.path().to_path_buf());
	}

	ret
}

// TODO: Optimize the kitten out of this
fn print_infotable(data: &Vec<MediaInfo>, maxes: (usize, usize, usize)) {
	let (mut num_vt, mut num_at, _) = maxes;

	let (vt, at, _) = &data.iter().fold((0, 0, 0),
							|acc, elem| {
								let (mut vt,mut at, _) = acc;
								if elem.video_tracks.len() > vt {
									vt = elem.video_tracks.len();
								};
								if elem.audio_tracks.len() > at {
									at = elem.audio_tracks.len();
								};
								(vt,at,0)
							});

	num_vt = min(num_vt, *vt);
	num_at = min(num_at, *at);

	let outlines: Vec<_> = data.iter()
					.map(|x| x.get_printline(num_vt, num_at, 0))
					.collect();

	let size = outlines.iter().fold(0, |acc, elem| if elem.len() > acc { elem.len() } else { acc });

	let mut padvec = vec![0; size];
	for o in &outlines {
		o.iter()
			.enumerate()
			.for_each(|(x, y)|
				if y.chars().count() > padvec[x] {
					padvec[x] = y.chars().count()
				});
	}

	for o in outlines.iter() {
		let tmp: Vec<_> = o.iter()
						.enumerate()
						.map(|(i,x)| format!("{1:0$}", padvec[i], x))
						.collect();
		println!("{}", tmp.join("  "));
	}
}


fn main() {
	// Probably use the macro instead, so we can have dynamic output based on build/runtime info
	let yaml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yaml).get_matches();
	let depth = {
		if !matches.is_present("recursive") && !matches.is_present("depth") {
			None
		}
		else if let Some(depth) = matches.value_of("depth") {
			Some(depth.parse::<usize>().unwrap())
		}
		else {
			// Build time parameterize this?
			Some(10)
		}
	};
	let maxes = (
		matches.value_of("video_tracks").unwrap_or("1").parse::<usize>().unwrap_or(1),
		matches.value_of("audio_tracks").unwrap_or("1").parse::<usize>().unwrap_or(1),
		0 // TODO: some day, actually implement subtitle tracks
	);

	let path = {
		Path::new(matches.value_of("DIR").unwrap_or("."))
	};
	if !path.exists() {
		println!("No such directory: {}", path.to_str().unwrap());
		return
	}

	let threads = matches.value_of("threads").unwrap_or("1").parse::<usize>().unwrap_or(1);
	let pool = ThreadPool::new(threads);
	let ret = Arc::new(Mutex::new(Vec::new()));

	for pa in get_media_paths(path, depth) {
		pool.execute({
			let clone = Arc::clone(&ret);
			move || {
				// TODO: check mode probably for sorting against dirs
				let out = MediaInfo::parse(String::from(pa.file_name().unwrap().to_str().unwrap()), get_mediainfo_output(pa)).unwrap();
				let mut v = clone.lock().unwrap();
				v.push(out);
		}});

	}

	pool.join();
	{
		let mut data = ret.lock().unwrap();
		let data = data.borrow_mut();

		// TODO: eventually figure out how to sort by the other columns
		data.sort();

		print_infotable(data, maxes);
	}

}
