use std::{fs, io, path::PathBuf, time::SystemTime};

#[derive(Clone, Debug)]
pub struct File {
  name: Option<String>,
  is_dir: bool,
  extension: Option<String>,
  created_at: SystemTime,
  children: Vec<File>,
  path: String
}

pub struct Manager {
  current: Option<PathBuf>,
  current_opened: Option<PathBuf>,
  files: Vec<File>,
}

impl Manager {
  pub fn new() -> Self {
    Self {
      current: None,
      current_opened: None,
      files: Vec::new()
    }
  }

  pub fn set_path(&mut self, path: String) -> &mut Self {
    self.current = Some(path.clone().into());
    self.current_opened = Some(path.into());
    self
  }

  pub fn path(&mut self) -> Option<PathBuf> {
    self.current_opened.clone()
  }

  pub fn load(&mut self) -> Result<&Self, io::Error> {
    if !self.current.is_some() {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "Current path can't be None"))
    }
    let files = self.parse_files(self.current.as_ref().unwrap().to_str().unwrap().to_string())?;
    self.files = files;
    Ok(self)
  }

  pub fn files(&self) -> Vec<File> {
    self.files.clone()
  }

  fn parse_files(&mut self, path: String) -> Result<Vec<File>, io::Error> {
    let mut files = Vec::new();

    let mut path_buf = PathBuf::from(path.clone());
    match path_buf.is_dir() {
      true => {
        match fs::read_dir(path) {
          Ok(r) => {
            for entry in r {
              let entry = entry?;
              let path = entry.path();
              let is_dir = path.is_dir();
              let name: Option<String> = path.file_name()
                .and_then(|os_str| os_str.to_str())
                .map(|s| s.to_string());

              let mut children = Vec::new();
              if is_dir && name.is_some() {
                self.current = Some(format!("{}\\{}", self.current.as_ref().unwrap().to_str().unwrap().to_string(), name.clone().unwrap()).into());
                children = self.parse_files(self.current.as_ref().unwrap().to_str().unwrap().to_string())?;
                self.current.as_mut().unwrap().pop();
              }
              let meta = entry.metadata()?;
              let created_at = meta.created()?;
              let extension = path.extension()
                .and_then(|os_str| os_str.to_str())
                .map(|s| s.to_string());
              let file = File::new(
                name,
                is_dir,
                extension,
                created_at,
                children,
                self.current.as_mut().unwrap().to_str().unwrap().to_string()
              );
              files.push(file);
            }
          },
          Err(e) => println!("Error opening folder: {}", e)
        }
      },
      false => {
        match fs::metadata(path.clone()) {
          Ok(meta) => {
            let name = path_buf.file_name()
              .and_then(|os_str| os_str.to_str())
              .map(|s| s.to_string());
            let extension = path_buf.extension()
                .and_then(|os_str| os_str.to_str())
                .map(|s| s.to_string());
            path_buf.pop();
            files.push(File::new(
              name,
              false,
              extension,
              meta.created()?,
              Vec::new(),
              path_buf.to_str().unwrap().to_string()
            ));
          },
          Err(e) => {
            println!("Error while opening file {}: {}", path_buf.file_name().unwrap().to_str().unwrap().to_string(), e);
          }
        }
      }
    }
    

    Ok(files)
  }
}

impl File {
  pub fn new(name: Option<String>, is_dir: bool, extension: Option<String>, created_at: SystemTime, children: Vec<File>, path: String) -> Self {
    Self {
      name,
      is_dir,
      extension,
      created_at,
      children,
      path
    }
  }

  pub fn name(&self) -> Option<String> {
    self.name.clone()
  }

  pub fn extension(&self) -> Option<String> {
    self.extension.clone()
  }

  pub fn is_dir(&self) -> bool {
    self.is_dir
  }

  pub fn children(&self) -> Vec<File> {
    self.children.clone()
  }

  pub fn path(&self) -> String {
    self.path.clone()
  }
}