mod scm;

use std::{mem::ManuallyDrop, path::PathBuf};

mod proto {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

#[repr(C)]
#[derive(Debug)]
pub struct Buffer {
    len: u32,
    data: *mut u8,
}

impl<T: prost::Message> From<T> for Buffer {
    fn from(value: T) -> Self {
        let mut bytes = ManuallyDrop::new(value.encode_to_vec());
        let data = bytes.as_mut_ptr();
        let len = bytes.len() as u32;
        Buffer { len, data }
    }
}

impl Buffer {
    #[allow(dead_code)]
    fn into_proto<T: prost::Message + Default>(self) -> Result<T, prost::DecodeError> {
        // SAFETY
        // protobuf has a fairly strict schema so overrunning or underrunning the byte
        // array will not cause any major issues, that is to say garbage in
        // garbage out
        let mut slice = unsafe { std::slice::from_raw_parts(self.data, self.len as usize) };
        T::decode(&mut slice)
    }
}

#[no_mangle]
pub extern "C" fn get_turbo_data_dir() -> Buffer {
    // note: this is _not_ recommended, but it the current behaviour go-side
    //       ideally we should use the platform specific convention
    //       (which we get from using ProjectDirs::from)
    let dirs =
        directories::ProjectDirs::from_path("turborepo".into()).expect("user has a home dir");

    let dir = dirs.data_dir().to_string_lossy().to_string();
    proto::TurboDataDirResp { dir }.into()
}

#[no_mangle]
pub extern "C" fn changed_files(buffer: Buffer) -> Buffer {
    let req: proto::ChangedFilesReq = buffer.into_proto().expect("buffer is valid protobuf");

    let commit_range = req.from_commit.as_deref().zip(req.to_commit.as_deref());
    let response = match scm::git::changed_files(
        PathBuf::from(req.repo_root),
        commit_range,
        req.include_untracked,
        req.relative_to.as_deref(),
    ) {
        Ok(files) => {
            let files: Vec<_> = files.into_iter().collect();
            proto::changed_files_resp::Response::Files(proto::ChangedFilesList { files })
        }
        Err(err) => proto::changed_files_resp::Response::Error(err.to_string()),
    };

    let resp = proto::ChangedFilesResp {
        response: Some(response),
    };
    resp.into()
}
