use crate::fs::file_tree::FileNode;
use crate::fs::fs_error::FsTestError;
use std::env::temp_dir;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct TmpTestFolder {
    path: PathBuf,
}

impl TmpTestFolder {
    pub fn new() -> Result<Self, FsTestError> {
        let path = temp_dir().as_path().join(Uuid::new_v4().to_string());
        fs::create_dir(&path)?;
        Ok(TmpTestFolder { path })
    }

    pub fn new_from_node(node: &FileNode) -> Result<Self, FsTestError> {
        let r = TmpTestFolder::new()?;
        r.write(node)?;
        Ok(r)
    }

    pub fn write(&self, node: &FileNode) -> Result<(), FsTestError> {
        node.write_to_path(&self.path)
    }

    pub fn read(&self) -> Result<FileNode, FsTestError> {
        FileNode::new_from_path(self.path.as_path())
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TmpTestFolder {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_test_folder() {
        let test_folder = TmpTestFolder::new().unwrap();

        assert!(test_folder.get_path().exists());
        assert!(test_folder.get_path().is_dir());
    }

    #[test]
    fn should_remove_test_folder_on_drop() {
        let saved_path: PathBuf;

        {
            let test_folder = TmpTestFolder::new().unwrap();

            assert!(test_folder.get_path().exists());
            assert!(test_folder.get_path().is_dir());

            saved_path = test_folder.get_path().to_path_buf();
        }
        assert!(!saved_path.exists());
    }

    #[test]
    fn should_write_file_node_to_tmp_folder() {
        let saved_path: PathBuf;

        {
            let test_folder = TmpTestFolder::new().unwrap();
            saved_path = test_folder.get_path().to_path_buf();

            test_folder
                .write(&FileNode::File {
                    name: "fr1".to_string(),
                    content: Vec::new(),
                })
                .unwrap();

            test_folder
                .write(&FileNode::File {
                    name: "fr2".to_string(),
                    content: "file content fr2".as_bytes().to_vec(),
                })
                .unwrap();

            test_folder
                .write(&FileNode::Dir {
                    name: "empty_dir".to_string(),
                    sub: vec![],
                })
                .unwrap();

            test_folder
                .write(&FileNode::Dir {
                    name: "d1".to_string(),
                    sub: vec![
                        FileNode::File {
                            name: "f1".to_string(),
                            content: "file content 1".as_bytes().to_vec(),
                        },
                        FileNode::File {
                            name: "f2".to_string(),
                            content: "file content 2".as_bytes().to_vec(),
                        },
                        FileNode::File {
                            name: "f3".to_string(),
                            content: "file content 3".as_bytes().to_vec(),
                        },
                        FileNode::Dir {
                            name: "d11".to_string(),
                            sub: vec![FileNode::File {
                                name: "f11".to_string(),
                                content: "file content 11".as_bytes().to_vec(),
                            }],
                        },
                    ],
                })
                .unwrap();

            assert!(test_folder.get_path().is_dir());
            assert!(test_folder.get_path().join("fr1").is_file());
            assert_eq!(fs::read_to_string(test_folder.get_path().join("fr1")).unwrap(), "");

            assert!(test_folder.get_path().join("fr2").is_file());
            assert_eq!(fs::read_to_string(test_folder.get_path().join("fr2")).unwrap(), "file content fr2");

            assert!(test_folder.get_path().join("d1").is_dir());

            assert!(test_folder.get_path().join("d1").join("f1").is_file());
            assert_eq!(fs::read_to_string(test_folder.get_path().join("d1").join("f1")).unwrap(), "file content 1");

            assert!(test_folder.get_path().join("d1").join("f2").is_file());
            assert_eq!(fs::read_to_string(test_folder.get_path().join("d1").join("f2")).unwrap(), "file content 2");

            assert!(test_folder.get_path().join("d1").join("f3").is_file());
            assert_eq!(fs::read_to_string(test_folder.get_path().join("d1").join("f3")).unwrap(), "file content 3");

            assert!(test_folder.get_path().join("d1").join("d11").is_dir());

            assert!(test_folder.get_path().join("d1").join("d11").join("f11").is_file());
            assert_eq!(fs::read_to_string(test_folder.get_path().join("d1").join("d11").join("f11")).unwrap(), "file content 11");

            assert!(test_folder.get_path().join("empty_dir").is_dir());
        }

        assert!(!saved_path.exists());
    }

    #[test]
    fn should_read_file_node_from_tmp_folder() {
        let saved_path: PathBuf;

        {
            let test_folder = TmpTestFolder::new().unwrap();
            saved_path = test_folder.get_path().to_path_buf();

            test_folder
                .write(&FileNode::File {
                    name: "fr1".to_string(),
                    content: Vec::new(),
                })
                .unwrap();

            test_folder
                .write(&FileNode::File {
                    name: "fr2".to_string(),
                    content: "file content fr2".as_bytes().to_vec(),
                })
                .unwrap();

            test_folder
                .write(&FileNode::Dir {
                    name: "empty_dir".to_string(),
                    sub: vec![],
                })
                .unwrap();

            test_folder
                .write(&FileNode::Dir {
                    name: "d1".to_string(),
                    sub: vec![
                        FileNode::File {
                            name: "f1".to_string(),
                            content: "file content 1".as_bytes().to_vec(),
                        },
                        FileNode::File {
                            name: "f2".to_string(),
                            content: "file content 2".as_bytes().to_vec(),
                        },
                        FileNode::File {
                            name: "f3".to_string(),
                            content: "file content 3".as_bytes().to_vec(),
                        },
                        FileNode::Dir {
                            name: "d11".to_string(),
                            sub: vec![FileNode::File {
                                name: "f11".to_string(),
                                content: "file content 11".as_bytes().to_vec(),
                            }],
                        },
                    ],
                })
                .unwrap();

            let tmp_node = test_folder.read().unwrap();

            assert_eq!(
                tmp_node
                    == FileNode::Dir {
                        name: test_folder.get_path().file_name().unwrap().to_str().unwrap().to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "fr1".to_string(),
                                content: Vec::new(),
                            },
                            FileNode::File {
                                name: "fr2".to_string(),
                                content: "file content fr2".as_bytes().to_vec(),
                            },
                            FileNode::Dir {
                                name: "d1".to_string(),
                                sub: vec![
                                    FileNode::File {
                                        name: "f1".to_string(),
                                        content: "file content 1".as_bytes().to_vec(),
                                    },
                                    FileNode::File {
                                        name: "f2".to_string(),
                                        content: "file content 2".as_bytes().to_vec(),
                                    },
                                    FileNode::File {
                                        name: "f3".to_string(),
                                        content: "file content 3".as_bytes().to_vec(),
                                    },
                                    FileNode::Dir {
                                        name: "d11".to_string(),
                                        sub: vec![FileNode::File {
                                            name: "f11".to_string(),
                                            content: "file content 11".as_bytes().to_vec(),
                                        }],
                                    },
                                ],
                            },
                            FileNode::Dir {
                                name: "empty_dir".to_string(),
                                sub: vec![],
                            },
                        ],
                    },
                true
            )
        }

        assert!(!saved_path.exists());
    }
}
