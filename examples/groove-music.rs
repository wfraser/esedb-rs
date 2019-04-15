//! This program is an example demonstrating how to use the ESEDB Rust library:
//!     https://github.com/wfraser/esedb-rs
//! It reads the database used by the Groove Music app on Windows and outputs a list of all tracks
//! in the collection of all profiles used by the current user.
//!
//! Copyright 2019 by William R. Fraser

extern crate esedb;
use esedb::*;

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

const APPDATA_PATH: &str = r"Packages\Microsoft.ZuneMusic_8wekyb3d8bbwe\LocalState\Database\";

struct EntPlatDb<'a> {
    // raw pointers have to be used here because Rust's lifetime system doesn't accomodate having
    // these three things in one struct (because destruction order would be unspecified).
    instance: *mut JetInstance,
    session: *mut JetSession<'a>,
    database: *mut JetDatabase<'a>,
}

impl<'a> EntPlatDb<'a> {
    pub fn get_database_root_path() -> PathBuf {
        let localappdata = env::var_os("localappdata").expect("%LOCALAPPDATA% missing");
        PathBuf::from(localappdata).join(APPDATA_PATH)
    }

    pub fn new(path: &Path) -> Result<EntPlatDb<'a>, JetError> {
        let pinstance: *mut JetInstance;
        let psession: *mut JetSession<'a>;
        let pdatabase: *mut JetDatabase<'a>;
        
        let mut instance = JetInstance::new();
        instance.set_int_parameter(JET_paramDatabasePageSize, 8192)?;
        instance.set_int_parameter(JET_paramEnableAdvanced, 1)?;
        instance.set_int_parameter(JET_paramEnableViewCache, 1)?;
        instance.set_int_parameter(JET_paramAccessDeniedRetryPeriod, 1000 /* ms */)?;
        instance.init_engine(&"esedb-rs-example".into())?;
        set_database_paths(&mut instance, path)?;
        instance.init()?;
        pinstance = Box::into_raw(Box::new(instance));

        let session = unsafe { (*pinstance).create_session()? };
        psession = Box::into_raw(Box::new(session));

        let file_path: WideString = path.join("EntClientDb.edb").as_os_str().into();
        let database = unsafe {
            (*psession).open_database(&file_path, DatabaseAccessMode::ReadOnly)?
        };
        pdatabase = Box::into_raw(Box::new(database));

        Ok(EntPlatDb {
            instance: pinstance,
            session: psession,
            database: pdatabase,
        })
    }

    pub fn get_artists(&self) -> Result<BTreeMap<u32, Artist>, JetError> {
        unsafe { get_artists(&*self.database) }
    }

    pub fn get_albums(&self) -> Result<BTreeMap<u32, Album>, JetError> {
        unsafe { get_albums(&*self.database) }
    }

    pub fn get_tracks(&self, start_id: Option<u32>) -> Result<BTreeMap<u32, Track>, JetError> {
        unsafe { get_tracks(&*self.database, start_id) }
    }
}

impl<'a> Drop for EntPlatDb<'a> {
    fn drop(&mut self) {
        use std::intrinsics::drop_in_place;
        unsafe {
            drop_in_place(self.database);
            drop_in_place(self.session);
            drop_in_place(self.instance);
        }
    }
}

fn set_database_paths(instance: &mut JetInstance, path: &Path) -> Result<(), JetError> {
    let wpath: WideString = path.join("").as_os_str().into();
    instance.set_string_parameter(JET_paramSystemPath, &wpath)?;
    instance.set_string_parameter(JET_paramLogFilePath, &wpath)?;
    instance.set_string_parameter(JET_paramTempPath, &wpath)?;
    instance.set_string_parameter(JET_paramAlternateDatabaseRecoveryPath, &wpath)?;
    Ok(())
}

pub struct Artist {
    pub name: String,
}

pub fn get_artists(db: &JetDatabase) -> Result<BTreeMap<u32, Artist>, JetError> {
    let mut map = BTreeMap::new();

    let table = db.open_table(&"tblPerson".into())?;
    let name_colid = table.get_column_id(&"Name".into())?;
    let id_colid = table.get_column_id(&"Id".into())?;
    loop {
        let artist_name = table.retrieve_wstring(name_colid)?;
        let artist_id: u32 = table.retrieve(id_colid)?;

        map.insert(artist_id, Artist { name: artist_name.to_string_lossy() });

        if let Err(e) = table.move_next() {
            if e.code == JET_errNoCurrentRecord {
                break;
            } else {
                return Err(e);
            }
        }
    }

    Ok(map)
}

pub struct Album {
    pub title: String,
    pub artist_id: u32,
}

fn get_albums(db: &JetDatabase) -> Result<BTreeMap<u32, Album>, JetError> {
    let mut map = BTreeMap::new();

    let table = db.open_table(&"tblAudioAlbum".into())?;
    let id_colid = table.get_column_id(&"Id".into())?;
    let title_colid = table.get_column_id(&"Title".into())?;
    let artist_id_colid = table.get_column_id(&"ArtistId".into())?;
    loop {
        let id = table.retrieve(id_colid)?;
        let title = table.retrieve_wstring(title_colid)?.to_string_lossy();
        let artist_id = table.retrieve(artist_id_colid)?;

        map.insert(id, Album {
            title,
            artist_id,
        });

        if let Err(e) = table.move_next() {
            if e.code == JET_errNoCurrentRecord {
                break;
            } else {
                return Err(e);
            }
        }
    }

    Ok(map)
}

pub struct Track {
    pub title: String,
    pub artist_id: u32,
    pub album_id: u32,
    pub collection_state: u8,
}

fn get_tracks(db: &JetDatabase, start_id: Option<u32>) -> Result<BTreeMap<u32, Track>, JetError> {
    let mut map = BTreeMap::new();

    let table = db.open_table(&"tblTrack".into())?;
    let id_colid = table.get_column_id(&"Id".into())?;
    let title_colid = table.get_column_id(&"Title".into())?;
    let artist_id_colid = table.get_column_id(&"ArtistId".into())?;
    let album_id_colid = table.get_column_id(&"AlbumId".into())?;
    let collection_state_colid = table.get_column_id(&"CollectionState".into())?;

    if let Some(id) = start_id {
        // no need to set an index; the ID is the primary key and therefore the default index.
        table.seek(SeekType::EqualOrGreater, &id)?;
    }

    loop {
        let id = table.retrieve(id_colid)?;
        let title = table.retrieve_wstring(title_colid)?.to_string_lossy();
        let artist_id = table.retrieve(artist_id_colid)?;
        let album_id = table.retrieve(album_id_colid)?;
        let collection_state = table.retrieve(collection_state_colid)?;

        map.insert(id, Track {
            title,
            artist_id,
            album_id,
            collection_state,
        });

        if let Err(e) = table.move_next() {
            if e.code == JET_errNoCurrentRecord {
                break;
            } else {
                return Err(e);
            }
        }
    }

    Ok(map)
}

fn main() {
    println!("This program lists all tracks in the database used by Groove Music on Windows");
    println!("as an example of how to use the ESEDB Rust library https://github.com/wfraser/esedb-rs");
    println!("--------------------------------------------------------------------------------------");

    let db_root_path = EntPlatDb::get_database_root_path();
    let mut folders = Vec::<PathBuf>::new();
    if let Ok(entries) = std::fs::read_dir(&db_root_path) {
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error enumerating database path: {:?}: {}", db_root_path, e);
                    continue;
                }
            };
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
            }
        }
    }

    for path in &folders {
        println!("User ID {:?}", path.file_name().unwrap());
        println!("--------------------------------------------------------------------------------------");
        let db = match EntPlatDb::new(path) {
            Ok(db) => db,
            Err(e) => {
                if e.code == JET_errFileAccessDenied {
                    println!("Failed to open the database. Probably Groove Music is running and using it.");
                } else {
                    println!("Failed to open the database: {}", e);
                }
                continue;
            }
        };

        let artists = db.get_artists().unwrap();
        let albums = db.get_albums().unwrap();
        let tracks = db.get_tracks(None).unwrap();

        for (track_id, track) in &tracks {
            if track.collection_state < 60 {
                // not in collection
                continue;
            }

            let title = &track.title;
            let artist = &artists[&track.artist_id].name;
            let album = &albums[&track.album_id].title;

            println!("{}: {} - {} - {}", track_id, artist, album, title);
        }
        println!();
    }
}
