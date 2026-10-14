use core::fmt;
use uefi::{CStr16, Status};
use uefi::proto::media::file::{File, FileAttribute, FileMode};

#[derive(Debug)]
pub struct SaveError {pub stage:&'static str,pub status:Status,pub cleanup_status:Option<Status>,pub file_number:Option<usize>}
impl SaveError {
    fn at(stage:&'static str,status:Status)->Self {Self{stage,status,cleanup_status:None,file_number:None}}
    fn for_file(stage:&'static str,status:Status,number:usize)->Self {Self{file_number:Some(number),..Self::at(stage,status)}}
}
impl fmt::Display for SaveError {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
        write!(f,"Save failed at {}: {:?}.",self.stage,self.status)?;
        if let Some(n)=self.file_number {write!(f," Attempted file number: {n:03}.")?;}
        if let Some(status)=self.cleanup_status {write!(f," Cleanup also failed: {status:?}; a partial or complete file may remain.")?;}
        Ok(())
    }
}

pub struct Report { bytes: [u8; 32768], used: usize, overflow: bool }
impl Report {
    pub fn text(&self) -> &str { core::str::from_utf8(&self.bytes[..self.used]).unwrap_or("Report encoding error") }
    pub fn checked_bytes(&self)->uefi::Result<&[u8]> {if self.overflow {Err(Status::BAD_BUFFER_SIZE.into())}else{Ok(&self.bytes[..self.used])}}
    pub fn new() -> Self { Self { bytes: [0; 32768], used: 0, overflow: false } }
    pub fn save(&self) -> uefi::Result<usize> {
        self.save_on(crate::media::Source::BootImage)
    }
    pub fn save_on(&self,source:crate::media::Source)->uefi::Result<usize> {
        self.save_on_diagnostic(source).map_err(|e|e.status.into())
    }
    pub fn save_on_diagnostic(&self,source:crate::media::Source)->Result<usize,SaveError> {
        if self.overflow {return Err(SaveError::at("report capacity",Status::BAD_BUFFER_SIZE));}
        save_numbered_diagnostic(&self.bytes[..self.used],[82,85,83,84],source)
    }
}

/// Writes a new numbered evidence file, preserving existing names.
pub fn save_numbered(bytes: &[u8], prefix: [u16;4]) -> uefi::Result<usize> {
        save_numbered_on(bytes,prefix,crate::media::Source::BootImage)
}

pub fn save_numbered_on(bytes:&[u8],prefix:[u16;4],source:crate::media::Source)->uefi::Result<usize> {
        save_numbered_diagnostic(bytes,prefix,source).map_err(|e|e.status.into())
}
fn save_numbered_diagnostic(bytes:&[u8],prefix:[u16;4],source:crate::media::Source)->Result<usize,SaveError> {
        let mut root = crate::media::open_root(source).map_err(|e|SaveError::at("open destination",e.status()))?;
        for number in 1..=999usize {
            let mut name = [92, 82, 85, 83, 84, 48, 48, 48, 46, 84, 88, 84, 0u16];
            name[1..5].copy_from_slice(&prefix);
            name[5] += (number / 100) as u16;
            name[6] += ((number / 10) % 10) as u16;
            name[7] += (number % 10) as u16;
            let name = CStr16::from_u16_with_nul(&name).map_err(|_| SaveError::at("filename",Status::INVALID_PARAMETER))?;
            match root.open(name, FileMode::Read, FileAttribute::empty()) {
                Ok(_) => continue,
                Err(e) if e.status() == Status::NOT_FOUND => {},
                Err(e) => return Err(SaveError::for_file("check existing filename",e.status(),number)),
            }
            let handle = root.open(name, FileMode::CreateReadWrite, FileAttribute::empty()).map_err(|e|SaveError::for_file("create file",e.status(),number))?;
            let mut file = handle.into_regular_file().ok_or(SaveError::for_file("file type",Status::UNSUPPORTED,number))?;
            let saved=file.write(bytes).map_err(|e|SaveError::for_file("write report",e.status(),number)).and_then(|_|file.flush().map_err(|e|SaveError::for_file("flush report",e.status(),number)));
            if let Err(mut error)=saved {
                // This handle was created only after NOT_FOUND. Never delete an
                // existing report. Preserve the original write/flush failure.
                if let Err(cleanup)=file.delete() {
                    error.cleanup_status=Some(cleanup.status());
                    uefi::println!("FAILED REPORT CLEANUP: {:?}",cleanup.status());
                }
                return Err(error);
            }
            return Ok(number);
        }
        Err(SaveError::at("numbered filenames exhausted",Status::VOLUME_FULL))
}
impl fmt::Write for Report {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if text.len() > self.bytes.len() - self.used { self.overflow = true; return Err(fmt::Error); }
        self.bytes[self.used..self.used + text.len()].copy_from_slice(text.as_bytes());
        self.used += text.len();
        Ok(())
    }
}
