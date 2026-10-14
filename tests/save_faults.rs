//! Execute production report.rs against an in-memory filesystem API double.
//! This tests control flow, not vendor firmware behavior or durable media writes.
extern crate self as uefi;
use std::{collections::BTreeMap,sync::Mutex};
use std::fmt::Write;
#[allow(non_camel_case_types)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Status {INVALID_PARAMETER,NOT_FOUND,UNSUPPORTED,BAD_BUFFER_SIZE,VOLUME_FULL,DEVICE_ERROR}
#[derive(Debug)] pub struct Error(Status);
impl Error {pub fn status(&self)->Status{self.0}}
impl From<Status> for Error {fn from(s:Status)->Self{Self(s)}}
pub type Result<T> = std::result::Result<T,Error>;
#[macro_export] macro_rules! println {($($t:tt)*)=>{std::println!($($t)*)};}
#[repr(transparent)]
pub struct CStr16([u16]);
impl CStr16 {
 pub fn from_u16_with_nul(v:&[u16])->std::result::Result<&Self,()> {
  if v.last()!=Some(&0)||v[..v.len()-1].contains(&0){return Err(());}
  Ok(unsafe{&*(v as *const [u16] as *const Self)})
 }
 fn name(&self)->String{String::from_utf16(&self.0[..self.0.len()-1]).unwrap()}
}
#[derive(Default)] struct State {fail:Option<&'static str>,cleanup_fail:bool,files:BTreeMap<String,Vec<u8>>,calls:Vec<&'static str>}
static STATE:Mutex<State>=Mutex::new(State{fail:None,cleanup_fail:false,files:BTreeMap::new(),calls:Vec::new()});
fn op(stage:&'static str)->Result<()> {
 let mut s=STATE.lock().unwrap();s.calls.push(stage);
 if s.fail==Some(stage){Err(Status::INVALID_PARAMETER.into())}else{Ok(())}
}
pub mod media {
 #[derive(Clone,Copy)] pub enum Source{BootImage}
 pub fn open_root(_:Source)->crate::Result<crate::proto::media::file::Directory>{crate::op("open destination")?;Ok(crate::proto::media::file::Directory)}
}
pub mod proto {pub mod media {pub mod file {
 use crate::*;
 pub enum FileMode{Read,CreateReadWrite}
 pub struct FileAttribute;
 impl FileAttribute{pub fn empty()->Self{Self}}
 pub struct Directory;
 pub struct Handle(String);
 pub struct RegularFile(String);
 pub trait File{fn flush(&mut self)->Result<()>;fn delete(self)->Result<()>;}
 impl Directory {
  pub fn open(&mut self,name:&CStr16,mode:FileMode,_:FileAttribute)->Result<Handle>{
   let name=name.name();
   match mode {
    FileMode::Read=>{op("check existing filename")?;if STATE.lock().unwrap().files.contains_key(&name){Ok(Handle(name))}else{Err(Status::NOT_FOUND.into())}},
    FileMode::CreateReadWrite=>{op("create file")?;STATE.lock().unwrap().files.insert(name.clone(),Vec::new());Ok(Handle(name))}
   }
  }
 }
 impl Handle{pub fn into_regular_file(self)->Option<RegularFile>{if STATE.lock().unwrap().fail==Some("file type"){None}else{Some(RegularFile(self.0))}}}
 impl RegularFile{pub fn write(&mut self,data:&[u8])->Result<()>{
  let result=op("write report");
  STATE.lock().unwrap().files.insert(self.0.clone(),if result.is_err(){data[..data.len()/2].to_vec()}else{data.to_vec()});result
 }}
 impl File for RegularFile {
  fn flush(&mut self)->Result<()>{op("flush report")}
  fn delete(self)->Result<()>{let mut s=STATE.lock().unwrap();s.calls.push("cleanup");if s.cleanup_fail{Err(Status::DEVICE_ERROR.into())}else{s.files.remove(&self.0);Ok(())}}
 }
}}}
#[path="../src/report.rs"] mod report;
// Each scenario spans multiple filesystem calls. Serialize whole scenarios,
// not just individual calls, so default parallel test execution is safe.
static SCENARIO: Mutex<()> = Mutex::new(());
fn reset(stage:Option<&'static str>,cleanup:bool){*STATE.lock().unwrap()=State{fail:stage,cleanup_fail:cleanup,..State::default()};}
fn sample()->report::Report{let mut r=report::Report::new();r.write_str("complete report\n").unwrap();r}
#[test] fn every_io_stage_preserves_original_error(){
 let _scenario=SCENARIO.lock().unwrap();
 for stage in ["open destination","check existing filename","create file","write report","flush report"] {
  reset(Some(stage),false);let e=sample().save_on_diagnostic(media::Source::BootImage).unwrap_err();
  assert_eq!(e.stage,stage);assert_eq!(e.status,Status::INVALID_PARAMETER);assert_eq!(e.cleanup_status,None);
  assert_eq!(e.file_number,if stage=="open destination"{None}else{Some(1)});
  assert!(!e.to_string().contains("Cleanup also failed"));
  let s=STATE.lock().unwrap();assert!(s.files.is_empty());assert_eq!(s.calls.contains(&"cleanup"),stage=="write report"||stage=="flush report");
 }
}
#[test] fn failed_flush_and_cleanup_can_leave_complete_readable_file(){
 let _scenario=SCENARIO.lock().unwrap();
 reset(Some("flush report"),true);let e=sample().save_on_diagnostic(media::Source::BootImage).unwrap_err();
 assert_eq!(e.stage,"flush report");assert_eq!(e.status,Status::INVALID_PARAMETER);assert_eq!(e.cleanup_status,Some(Status::DEVICE_ERROR));
 assert_eq!(e.file_number,Some(1));
 assert!(e.to_string().contains("Attempted file number: 001."));
 assert!(e.to_string().contains("Cleanup also failed: DEVICE_ERROR"));
 assert_eq!(STATE.lock().unwrap().files["\\RUST001.TXT"],b"complete report\n");
}
#[test] fn failed_write_cleanup_leaves_partial_file_but_next_attempt_preserves_it(){
 let _scenario=SCENARIO.lock().unwrap();
 reset(Some("write report"),true);assert!(sample().save().is_err());
 let prior=STATE.lock().unwrap().files["\\RUST001.TXT"].clone();
 {let mut s=STATE.lock().unwrap();s.fail=None;s.cleanup_fail=false;}
 assert_eq!(sample().save().unwrap(),2);
 assert_eq!(STATE.lock().unwrap().files["\\RUST001.TXT"],prior);
}
#[test] fn existing_names_and_decimal_boundaries_are_preserved(){
 let _scenario=SCENARIO.lock().unwrap();
 reset(None,false);
 for i in 1..=999 {assert_eq!(sample().save().unwrap(),i);}
 let e=sample().save_on_diagnostic(media::Source::BootImage).unwrap_err();
 assert_eq!(e.stage,"numbered filenames exhausted");assert_eq!(e.status,Status::VOLUME_FULL);
 let s=STATE.lock().unwrap();assert_eq!(s.files.len(),999);
 for n in [1,9,10,99,100,999]{assert_eq!(s.files[&format!("\\RUST{n:03}.TXT")],b"complete report\n");}
}
#[test] fn report_overflow_refuses_before_any_filesystem_operation(){
 let _scenario=SCENARIO.lock().unwrap();
 reset(None,false);let mut r=report::Report::new();r.write_str(&"x".repeat(32768)).unwrap();assert!(r.write_str("x").is_err());
 let e=r.save_on_diagnostic(media::Source::BootImage).unwrap_err();assert_eq!(e.stage,"report capacity");assert!(STATE.lock().unwrap().calls.is_empty());
}
#[test] fn non_regular_file_reports_its_stage(){
 let _scenario=SCENARIO.lock().unwrap();
 reset(Some("file type"),false);let e=sample().save_on_diagnostic(media::Source::BootImage).unwrap_err();assert_eq!(e.stage,"file type");assert_eq!(e.status,Status::UNSUPPORTED);
}
