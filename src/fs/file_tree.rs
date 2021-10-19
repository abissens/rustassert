use crate::fs::fs_error::FsTestError;
use crate::fs::fs_error::FsTestError::{NeedDir, NeedFile};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

pub enum FileNode {
    Dir { name: String, sub: Vec<FileNode> },
    File { name: String, open_options: Option<OpenOptions>, content: Vec<u8> },
}

impl PartialEq for FileNode {
    fn eq(&self, other: &Self) -> bool {
        return match self {
            FileNode::Dir { sub, name } => match other {
                FileNode::Dir { sub: o_sub, name: o_name } => {
                    if name != o_name {
                        return false;
                    }
                    if sub.len() != o_sub.len() {
                        return false;
                    }
                    let mut sub_r: Vec<&FileNode> = sub.iter().collect();
                    sub_r.sort_by_key(|f| f.get_name());

                    let mut o_sub_r: Vec<&FileNode> = o_sub.iter().collect();
                    o_sub_r.sort_by_key(|f| f.get_name());

                    for (i, e) in sub_r.iter().enumerate() {
                        if &o_sub_r[i] != e {
                            return false;
                        }
                    }
                    true
                }
                _ => false,
            },
            FileNode::File { content, name, .. } => match other {
                FileNode::File { content: o_content, name: o_name, .. } => name == o_name && content == o_content,
                _ => false,
            },
        };
    }
}

impl FileNode {
    pub fn new_dir(name: &str) -> Self {
        FileNode::Dir {
            name: String::from(name),
            sub: Vec::new(),
        }
    }

    pub fn new_file(name: &str, content: Vec<u8>) -> Self {
        FileNode::File {
            name: String::from(name),
            open_options: None,
            content,
        }
    }

    pub fn new_from_path(pb: &Path) -> Result<Self, FsTestError> {
        let mut f: FileNode;
        if pb.is_file() {
            f = FileNode::new_file("", vec![]);
        } else {
            f = FileNode::new_dir("");
        }
        f.feed_from_path(pb)?;
        Ok(f)
    }

    pub fn write_to_path(&self, root_dir: &Path) -> Result<(), FsTestError> {
        if !root_dir.is_dir() {
            return Err(NeedDir);
        }
        match self {
            FileNode::Dir { name, sub } => {
                let dir_path = root_dir.join(name);
                if !&dir_path.exists() {
                    fs::create_dir(&dir_path)?;
                }
                for sub_node in sub {
                    sub_node.write_to_path(&dir_path)?;
                }
                Ok(())
            }
            FileNode::File { name, content, open_options } => {
                let path = root_dir.join(name);
                let f = if !path.exists() {
                    File::create(path)?
                } else if let Some(oo) = open_options {
                    oo.open(path)?
                } else {
                    fs::OpenOptions::new().write(true).open(path)?
                };
                let mut f = BufWriter::new(&f);
                f.write_all(&content)?;
                f.flush()?;
                Ok(())
            }
        }
    }

    pub fn feed_from_path(&mut self, pb: &Path) -> Result<(), FsTestError> {
        match self {
            FileNode::Dir { ref mut name, sub } => {
                if !pb.is_dir() {
                    return Err(NeedDir);
                }
                *name = get_file_name(pb);
                for entry in fs::read_dir(pb)? {
                    let sub_path = entry?.path();
                    let mut s: FileNode;
                    if sub_path.is_file() {
                        s = FileNode::File {
                            name: "".to_string(),
                            content: vec![],
                            open_options: None,
                        };
                    } else {
                        s = FileNode::Dir { name: "".to_string(), sub: vec![] };
                    }
                    s.feed_from_path(&sub_path)?;
                    sub.push(s);
                }
                Ok(())
            }
            FileNode::File { ref mut name, ref mut content, .. } => {
                if !pb.is_file() {
                    return Err(NeedFile);
                }
                *name = get_file_name(pb);
                *content = fs::read(pb)?;
                Ok(())
            }
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            FileNode::Dir { name, .. } => &name,
            FileNode::File { name, .. } => &name,
        }
    }
}

fn get_file_name(pb: &Path) -> String {
    pb.file_name().unwrap_or_default().to_str().unwrap_or_default().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::io::{BufWriter, Write};
    use uuid::Uuid;

    #[test]
    fn file_nodes_are_comparable() {
        assert!(FileNode::new_dir("dir") == FileNode::new_dir("dir"));
        assert!(FileNode::new_dir("dir_1") != FileNode::new_dir("dir_2"));

        assert!(FileNode::new_file("f", Vec::new()) == FileNode::new_file("f", Vec::new()));
        assert!(FileNode::new_file("f_1", Vec::new()) != FileNode::new_file("f_2", Vec::new()));

        fn make_folder() -> FileNode {
            FileNode::Dir {
                name: "root".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "fr1".to_string(),
                        content: Vec::new(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "fr1".to_string(),
                        content: "file content r2".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "d1".to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "f1".to_string(),
                                content: "file content 1".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "f2".to_string(),
                                content: "file content 2".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "f3".to_string(),
                                content: "file content 3".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::Dir {
                                name: "d11".to_string(),
                                sub: vec![FileNode::File {
                                    name: "f11".to_string(),
                                    content: "file content 11".as_bytes().to_vec(),
                                    open_options: None,
                                }],
                            },
                        ],
                    },
                    FileNode::Dir {
                        name: "empty_dir".to_string(),
                        sub: vec![],
                    },
                ],
            }
        }

        assert!(make_folder() == make_folder());
    }

    #[test]
    fn write_to_path_should_return_err_when_writing_to_file() {
        let tmp_path = temp_dir();

        File::create(tmp_path.join("some_file")).unwrap();

        let file_name = Uuid::new_v4().to_string();
        let file_node = FileNode::File {
            name: file_name,
            content: "file content".as_bytes().to_vec(),
            open_options: None,
        };

        let result = file_node.write_to_path(&tmp_path.join("some_file"));
        assert!(matches!(result, Err(NeedDir)));
    }

    #[test]
    fn write_to_path_should_ignore_existing_folders() {
        let tmp_path = temp_dir();
        let folder_name = Uuid::new_v4().to_string();
        let tmp_folder_path = tmp_path.join(&folder_name);
        let file_node = FileNode::Dir { name: folder_name, sub: vec![] };

        file_node.write_to_path(&tmp_path).unwrap();
        file_node.write_to_path(&tmp_path).unwrap();

        fs::remove_dir(tmp_folder_path).unwrap();
    }

    #[test]
    fn write_to_path_should_write_single_file() {
        let tmp_path = temp_dir();
        let folder_root_name = Uuid::new_v4().to_string();

        let root_dir = FileNode::Dir {
            name: folder_root_name.clone(),
            sub: vec![
                FileNode::File {
                    name: "fr1".to_string(),
                    content: Vec::new(),
                    open_options: None,
                },
                FileNode::File {
                    name: "fr2".to_string(),
                    content: "file content fr2".as_bytes().to_vec(),
                    open_options: None,
                },
                FileNode::Dir {
                    name: "d1".to_string(),
                    sub: vec![
                        FileNode::File {
                            name: "f1".to_string(),
                            content: "file content 1".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::File {
                            name: "f2".to_string(),
                            content: "file content 2".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::File {
                            name: "f3".to_string(),
                            content: "file content 3".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::Dir {
                            name: "d11".to_string(),
                            sub: vec![FileNode::File {
                                name: "f11".to_string(),
                                content: "file content 11".as_bytes().to_vec(),
                                open_options: None,
                            }],
                        },
                    ],
                },
                FileNode::Dir {
                    name: "empty_dir".to_string(),
                    sub: vec![],
                },
            ],
        };

        root_dir.write_to_path(&tmp_path).unwrap();

        FileNode::Dir {
            name: folder_root_name.clone(),
            sub: vec![FileNode::Dir {
                name: "d1".to_string(),
                sub: vec![FileNode::File {
                    name: "f4".to_string(),
                    content: "file content 4".as_bytes().to_vec(),
                    open_options: None,
                }],
            }],
        }
        .write_to_path(&tmp_path)
        .unwrap();

        root_dir.write_to_path(&tmp_path).unwrap();

        assert!(tmp_path.join(&folder_root_name).is_dir());
        assert!(tmp_path.join(&folder_root_name).join("fr1").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("fr1")).unwrap(), "");

        assert!(tmp_path.join(&folder_root_name).join("fr2").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("fr2")).unwrap(), "file content fr2");

        assert!(tmp_path.join(&folder_root_name).join("d1").is_dir());

        assert!(tmp_path.join(&folder_root_name).join("d1").join("f1").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("f1")).unwrap(), "file content 1");

        assert!(tmp_path.join(&folder_root_name).join("d1").join("f2").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("f2")).unwrap(), "file content 2");

        assert!(tmp_path.join(&folder_root_name).join("d1").join("f3").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("f3")).unwrap(), "file content 3");

        assert!(tmp_path.join(&folder_root_name).join("d1").join("f4").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("f4")).unwrap(), "file content 4");

        assert!(tmp_path.join(&folder_root_name).join("d1").join("d11").is_dir());

        assert!(tmp_path.join(&folder_root_name).join("d1").join("d11").join("f11").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("d11").join("f11")).unwrap(), "file content 11");

        assert!(tmp_path.join(&folder_root_name).join("empty_dir").is_dir());

        fs::remove_dir_all(tmp_path.join(&folder_root_name)).unwrap();
    }

    #[test]
    fn write_to_path_should_write_empty_folder() {
        let tmp_path = temp_dir();
        let folder_name = Uuid::new_v4().to_string();
        let tmp_folder_path = tmp_path.join(&folder_name);
        let file_node = FileNode::Dir { name: folder_name, sub: vec![] };

        file_node.write_to_path(&tmp_path).unwrap();

        assert!(tmp_folder_path.read_dir().unwrap().next().is_none());

        fs::remove_dir(tmp_folder_path).unwrap();
    }

    #[test]
    fn write_to_path_should_overwrite_file_when_exists() {
        let tmp_path = temp_dir();
        let file_name = Uuid::new_v4().to_string();
        let file_node = FileNode::File {
            name: file_name.clone(),
            content: "file content".as_bytes().to_vec(),
            open_options: None,
        };

        file_node.write_to_path(&tmp_path).unwrap();

        assert!(tmp_path.join(&file_name).is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&file_name)).unwrap(), "file content");

        let file_node = FileNode::File {
            name: file_name.clone(),
            content: "file content updated".as_bytes().to_vec(),
            open_options: None,
        };

        file_node.write_to_path(&tmp_path).unwrap();

        assert!(tmp_path.join(&file_name).is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&file_name)).unwrap(), "file content updated");

        fs::remove_file(tmp_path.join(&file_name)).unwrap();
    }

    #[test]
    fn write_to_path_should_use_input_options() {
        let tmp_path = temp_dir();
        let file_name = Uuid::new_v4().to_string();
        let file_node = FileNode::File {
            name: file_name.clone(),
            content: "file content.".as_bytes().to_vec(),
            open_options: None,
        };

        file_node.write_to_path(&tmp_path).unwrap();

        assert!(tmp_path.join(&file_name).is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&file_name)).unwrap(), "file content.");

        let file_node = FileNode::File {
            name: file_name.clone(),
            content: " appended content".as_bytes().to_vec(),
            open_options: Some({
                let mut options = OpenOptions::new();
                options.append(true);
                options
            }),
        };

        file_node.write_to_path(&tmp_path).unwrap();

        assert!(tmp_path.join(&file_name).is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&file_name)).unwrap(), "file content. appended content");

        fs::remove_file(tmp_path.join(&file_name)).unwrap();
    }

    #[test]
    fn write_to_path_should_write_empty_folder_hierarchy() {
        let tmp_path = temp_dir();
        let folder_root_name = Uuid::new_v4().to_string();

        let root_dir = FileNode::Dir {
            name: folder_root_name.clone(),
            sub: vec![
                FileNode::File {
                    name: "fr1".to_string(),
                    content: Vec::new(),
                    open_options: None,
                },
                FileNode::File {
                    name: "fr2".to_string(),
                    content: "file content fr2".as_bytes().to_vec(),
                    open_options: None,
                },
                FileNode::Dir {
                    name: "d1".to_string(),
                    sub: vec![
                        FileNode::File {
                            name: "f1".to_string(),
                            content: "file content 1".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::File {
                            name: "f2".to_string(),
                            content: "file content 2".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::File {
                            name: "f3".to_string(),
                            content: "file content 3".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::Dir {
                            name: "d11".to_string(),
                            sub: vec![FileNode::File {
                                name: "f11".to_string(),
                                content: "file content 11".as_bytes().to_vec(),
                                open_options: None,
                            }],
                        },
                    ],
                },
                FileNode::Dir {
                    name: "empty_dir".to_string(),
                    sub: vec![],
                },
            ],
        };

        root_dir.write_to_path(&tmp_path).unwrap();

        assert!(tmp_path.join(&folder_root_name).is_dir());
        assert!(tmp_path.join(&folder_root_name).join("fr1").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("fr1")).unwrap(), "");

        assert!(tmp_path.join(&folder_root_name).join("fr2").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("fr2")).unwrap(), "file content fr2");

        assert!(tmp_path.join(&folder_root_name).join("d1").is_dir());

        assert!(tmp_path.join(&folder_root_name).join("d1").join("f1").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("f1")).unwrap(), "file content 1");

        assert!(tmp_path.join(&folder_root_name).join("d1").join("f2").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("f2")).unwrap(), "file content 2");

        assert!(tmp_path.join(&folder_root_name).join("d1").join("f3").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("f3")).unwrap(), "file content 3");

        assert!(tmp_path.join(&folder_root_name).join("d1").join("d11").is_dir());

        assert!(tmp_path.join(&folder_root_name).join("d1").join("d11").join("f11").is_file());
        assert_eq!(fs::read_to_string(tmp_path.join(&folder_root_name).join("d1").join("d11").join("f11")).unwrap(), "file content 11");

        assert!(tmp_path.join(&folder_root_name).join("empty_dir").is_dir());

        fs::remove_dir_all(tmp_path.join(&folder_root_name)).unwrap();
    }

    #[test]
    fn feed_from_path_should_feed_from_written_node() {
        let tmp_path = temp_dir();
        let folder_root_name = Uuid::new_v4().to_string();

        let root_dir = FileNode::Dir {
            name: folder_root_name.clone(),
            sub: vec![
                FileNode::File {
                    name: "fr1".to_string(),
                    content: Vec::new(),
                    open_options: None,
                },
                FileNode::File {
                    name: "fr2".to_string(),
                    content: "file content fr2".as_bytes().to_vec(),
                    open_options: None,
                },
                FileNode::Dir {
                    name: "d1".to_string(),
                    sub: vec![
                        FileNode::File {
                            name: "f1".to_string(),
                            content: "file content 1".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::File {
                            name: "f2".to_string(),
                            content: "file content 2".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::File {
                            name: "f3".to_string(),
                            content: "file content 3".as_bytes().to_vec(),
                            open_options: None,
                        },
                        FileNode::Dir {
                            name: "d11".to_string(),
                            sub: vec![FileNode::File {
                                name: "f11".to_string(),
                                content: "file content 11".as_bytes().to_vec(),
                                open_options: None,
                            }],
                        },
                    ],
                },
                FileNode::Dir {
                    name: "empty_dir".to_string(),
                    sub: vec![],
                },
            ],
        };

        root_dir.write_to_path(&tmp_path).unwrap();

        let result = FileNode::new_from_path(&tmp_path.join(&folder_root_name).as_path()).unwrap();

        assert!(result == root_dir);
    }

    #[test]
    fn feed_from_path_should_feed_single_file() {
        let f_name = Uuid::new_v4().to_string();
        let f_path = temp_dir().as_path().join(&f_name);
        let f = File::create(&f_path).unwrap();

        let mut f = BufWriter::new(f);
        f.write_all("Some data!".as_bytes()).unwrap();
        f.flush().unwrap();

        let file_node = FileNode::new_from_path(&f_path.as_path()).unwrap();

        assert!(file_node == FileNode::new_file(&f_name, "Some data!".as_bytes().to_vec()));

        fs::remove_file(f_path).unwrap();
    }

    #[test]
    fn feed_from_path_should_feed_empty_folder() {
        let f_name = Uuid::new_v4().to_string();
        let f_path = temp_dir().as_path().join(&f_name);
        fs::create_dir(&f_path).unwrap();

        let dir_node = FileNode::new_from_path(&f_path.as_path()).unwrap();

        assert!(dir_node == FileNode::new_dir(&f_name));

        fs::remove_dir(&f_path).unwrap();
    }
}
