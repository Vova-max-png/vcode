use std::{collections::btree_map::Range, fs::File, io::{self, BufWriter}, ops::{Bound, RangeBounds}, path::PathBuf};
use ropey::{Rope, RopeSlice};

use uuid::Uuid;

#[derive(Clone)]
struct EditorInstance {
  id: String,
  content: Rope,
  path: String,
  saved: bool
}

pub struct Editor {
  pub auto_save: bool,
  editor_instances: Vec<EditorInstance>,
  active_instance_id: Option<String>,
  pub current_content: Rope
}

impl Editor {
  pub fn new() -> Self {
    Self {
      auto_save: false,
      editor_instances: Vec::new(),
      active_instance_id: None,
      current_content: Rope::new()
    }
  }

  // Function used to create new instance of editor.
  // Firstly, check whether there is already an instance with such path
  // to prevent creating many instances for one file
  pub fn new_instance(&mut self, path: String) -> Result<(), io::Error> {
    match self.instance_by_path(path.clone()) {
      Some(i) => { self.set_active_instance(i.id).unwrap(); },
      None => {
        let mut instance = EditorInstance::new(path.clone());
        instance.parse(path.clone())?;
        self.active_instance_id = Some(instance.id());
        self.editor_instances.push(instance);
        self.current_content = self.current_instance()?.unwrap().clone().content;
      }
    };
    Ok(())
  }

  // Function to manage current content and active_instance_id 
  // by selecting already existing instance by it's id
  fn set_active_instance(&mut self, instance_id: String) -> Result<(), io::Error> {
    self.active_instance_id = Some(instance_id);
    self.current_content = self.current_instance()?.unwrap().clone().content;
    Ok(())
  }

  // Function that returns all opened instances 
  pub fn instances_data(&mut self) -> Result<Vec<(String, String, bool)>, io::Error> {
    let instances = self.editor_instances.clone();
    let mut data = Vec::new();
    for i in instances {
      let path = PathBuf::from(i.path.clone());
      let name = path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.to_string()).unwrap_or("Unknown file".to_string());
      let saved = i.saved;
      data.push((name, i.path, saved));
    }
    Ok(data)
  }

  // Function to find instances by it's path.
  // Used to check whether there is already instance with such path
  // to prevent creating many instances for one file
  fn instance_by_path(&self, path: String) -> Option<EditorInstance> {
    match self.editor_instances.iter().find(|i| i.path == path) {
      Some(i) => return Some(i.clone()),
      None => return None
    }
  }

  // Function used to update current instance's content
  pub fn update_instance_content<R>(&mut self, range: R, buf: String) -> Result<(), io::Error>
  where 
    R: RangeBounds<usize> + Clone
    {
    let auto_save = self.auto_save;
    self.current_content.remove(range.clone());
    let range_start = match range.start_bound() {
      Bound::Included(&s) => s,
      Bound::Excluded(&s) => s + 1,
      Bound::Unbounded => 0
    };
    self.current_content.insert(range_start, &buf);
    let current_content = self.current_content.clone();
    let current_instance = match self.current_instance()?{
      Some(i) => i,
      None => return Err(io::Error::new(io::ErrorKind::NotFound, "No instance selected yet!"))
    };
    current_instance.content = current_content;
    match auto_save {
      true => { current_instance.save()?; },
      false => { current_instance.set_unsaved(); },
    }
    Ok(())
  }

  // Function that calls current instance's save function
  // that saves it's content accordingly to it's path
  pub fn save_current_instance(&mut self) ->Result<(), io::Error> {
    let current_instance = match self.current_instance()? {
      Some(i) => i,
      None => return Err(io::Error::new(io::ErrorKind::NotFound, "There is no active instance to be saved!"))
    };
    current_instance.save()?;
    Ok(())
  }

  // Function used to retrieve current instance from vector array by currently opened instance's id
  fn current_instance(&mut self) -> Result<Option<&mut EditorInstance>, io::Error> {
    let instance = self.editor_instances.iter_mut().find(|i| i.id == self.active_instance_id.as_deref().unwrap());
    match instance {
      Some(i) => Ok(Some(i)),
      None => Ok(None)
    }
  }
}

impl EditorInstance {
  pub fn new(path: String) -> Self {
    let instance_id = format!("{}-{}", path, Uuid::new_v4());
    Self {
      id: instance_id,
      content: Rope::new(),
      path,
      saved: true
    }
  }

  pub fn parse(&mut self, path: String) -> Result<&Self, io::Error> {
    self.content = Rope::from_reader(std::fs::File::open(path)?)?;
    Ok(self)
  }

  pub fn save(&mut self) -> Result<&mut Self, io::Error> {
    println!("Writing: {}", self.content.to_string());
    self.content.write_to(BufWriter::new(File::create(self.path.clone())?))?;
    self.saved = true;
    Ok(self)
  }

  pub fn set_unsaved(&mut self) {
    self.saved = false;
  }

  pub fn id(&self) -> String {
    self.id.clone()
  }
}