use clap::Parser;
use frankenstein::Api;
use frankenstein::TelegramApi;
use frankenstein::SendMessageParams;
use frankenstein::GetUpdatesParams;
use frankenstein::ChatId;
use frankenstein::Message;
use frankenstein::api_params::File;
use frankenstein::api_params::InputFile;
use frankenstein::api_params::SendDocumentParams;
use frankenstein::objects::UpdateContent;
use frankenstein::objects::AllowedUpdate;
use std::fs::DirEntry;
use std::{fs, thread, time};

mod langs;

/*
 * 4096 is the max character length
 * of the text parameter in the sendMessage
 * method of the telegram bot api
 * (char>=1byte )
 * see: https://core.telegram.org/bots/api#sendmessage
*/
const MAX_TEXT_LEN: usize = 4096;

struct ErrNotFound {
	message: String
}

#[derive(Parser, Debug)]
struct Args {
	#[arg(short, long, help = "Telegram Bot API Token")]
	token: String,
	#[arg(short, long, help = "Absolute Path To Folder With PDF Files")]
	songs_path: String,
	#[arg(short, long, default_value = "en", help = "Language that the bot speaks: 'en', 'de' or 'md'")]
	lang: String,
	#[arg(short = 'f', long, help = "Absolute Path To Search File")]
	search_file: Option<String>
}

struct IncomingTextMsg {
	api: Api,
	msg: Option<Message>,
	strings: langs::Strings,
	songs_path: String,
	search_file: Option<String>
}

enum OutgoingTextMsg {
	DirEntry(Vec<DirEntry>),
	String(Vec<String>)
}

struct FindSongArgs {
	songs_path: String,
	strings: langs::Strings,
	search_string: String,
	search_type: SearchType,
	search_file: String
}

#[derive(Debug)]
enum SearchType {
	Title,
	FullText
}

struct SearchResult {
	ss_in_title: Vec<String>,
	ss_in_lyrics: Vec<String>
}

fn main() {
	let args = Args::parse();
	let api = Api::new(&args.token.as_str());
	let songs_path: String = add_ending_slash(args.songs_path);
	let mut incoming_text_msg = IncomingTextMsg {
		api: api.clone(),
		msg: None,
		strings: langs::Strings::new(args.lang, songs_path.clone()),
		songs_path: songs_path.clone(),
		search_file: args.search_file.clone()
	};
	let mut updates_params = GetUpdatesParams::builder()
		.allowed_updates(vec![AllowedUpdate::Message])
		.build();
	loop {
		let dur = time::Duration::from_millis(500);
		thread::sleep(dur);
		let result = TelegramApi::get_updates(&api, &updates_params);
		match result {
			Ok(val) => {
				for update in &val.result {
					updates_params.offset = Some(i64::from(update.update_id) + 1);
					match &update.content {
						UpdateContent::Message(msg) => {
							if msg.text.is_some() {
								incoming_text_msg.msg = Some(msg.clone());
								handle_text_message(&incoming_text_msg);
							}
						},
						_ => {}
					}
				}
			},
			Err(_err) => {
				println!("Error receiving updates from Telegram Bot API.");
			}
		}
	}
}

fn add_ending_slash(path: String) -> String {
	if !path.ends_with("/") {
		let mut new_path = path.to_owned();
		new_path.push_str("/");
		return new_path;
	}
	else {
		return path;
	}
}

fn handle_text_message(args: &IncomingTextMsg) {
	let mut find_song_args = FindSongArgs {
		search_string: String::new(),
		songs_path: args.songs_path.clone(),
		strings: args.strings.clone(),
		search_type: SearchType::Title,
		search_file: String::new()
	};
	if args.search_file.as_ref().is_some() {
		let search_file = args.search_file.as_ref().unwrap();
		if fs::File::open(search_file).is_ok() {
			find_song_args.search_type = SearchType::FullText;
			find_song_args.search_file =  search_file.to_string();
		}
	}
	let msg = args.msg.clone().unwrap();
	let text: &str = msg.text.as_ref().unwrap();
	let chat_id: u64 = msg.from.as_ref().unwrap().id;
	let mut params = SendMessageParams::builder()
		.chat_id(ChatId::Integer(chat_id.try_into().unwrap()))
		.text("")
		.build();
	match text {
		"/start" => {
			params.text = (args.strings.start_msg).to_string();
			send_message(&args.api, &mut params);
		},
		"/list" => {
			let songs = get_songs(&args.songs_path, None);
			params.text = form_msg(OutgoingTextMsg::DirEntry(songs));
			send_message(&args.api, &mut params);
		},
		_ => {
			if text.starts_with("/") {
				for name in langs::get_folder_names(&args.songs_path) {
					if text == "/".to_owned() + name.as_str() {
						let songs = get_songs(&args.songs_path, Some(&name));
						params.text = form_msg(OutgoingTextMsg::DirEntry(songs));
						send_message(&args.api, &mut params);
						return;
					}
				}
				let len = text.as_bytes().len();
				find_song_args.search_string = text[1..len].to_string();
				match title_search(&find_song_args) {
					Ok(files) => {
						let file = files.get(0);
						let input_file = InputFile::builder()
							.path(file.unwrap().path())
							.build();
						let send_document_params = SendDocumentParams::builder()
							.chat_id(ChatId::Integer(chat_id.try_into().unwrap()))
							.document(File::InputFile(input_file))
							.build();
						send_document(&args.api, &send_document_params);
					},
					Err(err) => {
						println!("{}", err.message);
						params.text = (args.strings.song_not_found).to_string();
						send_message(&args.api, &mut params);
					}
				}
			}
			else {
				find_song_args.search_string = text.to_string();
				match find_song_args.search_type {
					SearchType::Title => {
						match title_search(&find_song_args) {
							Ok(files) => {
								params.text = form_msg(OutgoingTextMsg::DirEntry(files));
								send_message(&args.api, &mut params);
							},
							Err(err) => {
								println!("{}", err.message);
								params.text = (args.strings.song_not_found).to_string();
								send_message(&args.api, &mut params);
							}
						}
					},
					SearchType::FullText => {
						match full_text_search(&find_song_args) {
							Ok(search_result) => {
								let ss_in_title = form_msg(OutgoingTextMsg::String(search_result.ss_in_title));
								let ss_in_lyrics = form_msg(OutgoingTextMsg::String(search_result.ss_in_lyrics));
								params.text.push_str(&ss_in_title);
								params.text.push_str(&ss_in_lyrics);
								send_message(&args.api, &mut params);
							},
							Err(err) => {
								println!("{}", err.message);
								params.text = (args.strings.song_not_found).to_string();
								send_message(&args.api, &mut params);
							}
						}
					}
				}
			}
		}
	}
}

fn send_document(api: &Api, params: &SendDocumentParams) {
	let result = api.send_document(params);
	match result {
		Err(err) => {
			println!("send_document failed.");
			dbg!(err);
		},
		Ok(_res) => {}
	}
}

fn send_message(api: &Api, params: &mut SendMessageParams) {
	let text_len = params.text.chars().count();
	let msg_count = text_len as f64 / MAX_TEXT_LEN as f64;
	if msg_count <= 1.0 {
		let result = api.send_message(params);
		match result {
			Err(err) => {
				println!("send_message failed.");
				dbg!(err);
			},
			Ok(_res) => {}
		}
	} else {
		let mut text: String = params.text.clone();
		let mut part: &str;
		loop {
			if text.chars().count() > MAX_TEXT_LEN {
				match find_last_line_break(text.clone()) {
					Ok(index) => {
						part = &text[..index];
						params.text = part.to_string();
						let result = api.send_message(params);
						match result {
							Err(err) => {
								println!("send_message failed.");
								dbg!(err);
							},
							Ok(_res) => {}
						}
						text = text[index+1..].to_string();
					},
					Err(_) => {
						println!("Dude, there's no line break. Deal with it.");
					}
				}
			} else {
				params.text = text;
				let result = api.send_message(params);
				match result {
					Err(err) => {
						println!("send_message failed.");
						dbg!(err);
					},
					Ok(_res) => {}
				}
				break;
			}
		}
	}
}

fn find_last_line_break(text: String) -> Result<usize, usize> {
	let mut i: usize = MAX_TEXT_LEN as usize;
	loop {
		if i == 0 {
			return Err(i);
		}
		match text.chars().nth(i) {
			Some(c) => {
				if c == '\n' {
					return Ok(i);
				}
			},
			None => {
				println!("nth error");
			}
		}
		i -= 1;
	}
}

fn form_msg(songs: OutgoingTextMsg) -> String {
	let mut message = String::new();
	match songs {
		OutgoingTextMsg::DirEntry(songs) => {
			for song in songs {
				let file_name = song.file_name();
				let filename = file_name.to_str().unwrap();
				let s: Vec<&str> = filename.split(".").collect();
				let name = s.get(0).unwrap();
				let mut command: String = "/".to_string();
				command.push_str(name);
				command.push_str("\n");
				message.push_str(command.as_str());
			}
		},
		OutgoingTextMsg::String(songs) => {
			for song in songs {
				let mut command = String::from("/");
				command.push_str(&song);
				command.push_str("\n");
				message.push_str(command.as_str());
			}
		}
	}
	return message;
}

fn title_search(args: &FindSongArgs) -> Result<Vec<DirEntry>, ErrNotFound> {
	let mut result: Vec<DirEntry> = vec![];
	let mut filename: String;
	let ss = args.strings.format(&args.search_string).to_lowercase();
	for file in get_songs(&args.songs_path, None) {
		filename = file.file_name().to_str().unwrap().to_string();
		let f: Vec<&str> = filename.split(".").collect();
		let mut name: String = f.get(0).unwrap().to_string();
		name = name.to_lowercase();
		if name == ss {
			let mut one = vec![file];
			one.append(&mut result);
			result = one;
		} else if name.contains(&ss) {
			result.push(file);
		}
	}
	if result.len() > 0 {
		return Ok(result);
	} else {
		return Err(ErrNotFound { message: String::from("Didn't find any song.") });
	}
}

fn full_text_search(args: &FindSongArgs) -> Result<SearchResult, ErrNotFound> {
	let mut ss_in_title: Vec<String> = vec![];
	let mut ss_in_lyrics: Vec<String> = vec![];
	let ss = prepare_for_search(&args.search_string);
	let content = fs::read_to_string(&args.search_file).unwrap();
	for line in content.lines() {
		let s_line: Vec<&str> = line.split(':').collect();
		let name = s_line.get(0).unwrap();
		let song_title = s_line.get(1).unwrap();
		let song_lyrics = s_line.get(2).unwrap();
		if song_title.contains(&ss) {
			ss_in_title.push(name.to_string());
		}
		else if song_lyrics.contains(&ss) {
			ss_in_lyrics.push(name.to_string());
		}
	}
	if ss_in_title.len() == 0 && ss_in_lyrics.len() == 0 {
		return Err(ErrNotFound { message: String::from("Didn't find any song.") });
	} else {
		return Ok(SearchResult { ss_in_title, ss_in_lyrics });
	}
}

fn prepare_for_search(string: &String) -> String {
	let mut res = String::new();
	// let mut is_last_line_break = false;
	for c in string.chars() {
		if c.is_alphabetic() {
			res.push(c);
			// is_last_line_break = false;
		}
		/* if c == '\n' || c == ' ' {
			if !is_last_line_break {
				res.push(' ');
				is_last_line_break = true;
			}
		} */
	}
	res = res.to_lowercase();
	return res;
}

fn get_songs(songs_path: &String, folder_name: Option<&String>) -> Vec<DirEntry> {
	match folder_name {
		Some(name) => {
			return get_files_recursive(&(songs_path.to_owned() + name));
		},
		None => {
			return get_files_recursive(songs_path);
		}
	}
}

fn get_files_recursive(folder_path: &String) -> Vec<DirEntry> {
	let path = fs::read_dir(folder_path);
	let mut is_dir: bool;
	let mut songs: Vec<DirEntry> = vec![];
	match path {
		Ok(read_dir) => {
			for r in read_dir {
				match r {
					Ok(dir_entry) => {
						is_dir = dir_entry.file_type().unwrap().is_dir();
						if is_dir {
							let path = dir_entry.path().to_str().unwrap().to_string();
							for song in get_files_recursive(&path) {
								songs.push(song);
							}
						} else {
							songs.push(dir_entry);
						}
					},
					Err(err) => {
						println!("Cannot access filepath.");
						dbg!(err);
					}
				}
			}
		},
		Err(_) => {
			println!("Cannot open/read or what ever the path {}.", folder_path);
		}
	}
	songs.sort_by_key(|name| name.file_name().into_string().unwrap().to_lowercase());
	return songs;
}
