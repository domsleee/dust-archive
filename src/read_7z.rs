use du_dust::DisplayNode;
use sevenz_rust::{Password, SevenZReader};
use std::fs::File;
use std::io::{BufReader, Error as IoError};
use std::path::{Path, PathBuf};

// Compressed sizes are 0 for 7z archives in a block format
// https://github.com/dyz1990/sevenz-rust/issues/49
pub fn read_7z(
    zip_file: &PathBuf,
    depth: usize,
    use_actual_size: bool,
) -> Result<DisplayNode, IoError> {
    let file = File::open(zip_file)?;
    let file_size = file.metadata()?.len();
    let reader = BufReader::new(file);
    let mut archive = SevenZReader::new(reader, file_size, Password::empty()).map_err(|e| {
        IoError::new(
            std::io::ErrorKind::Other,
            format!("Invalid 7z archive: {}", e),
        )
    })?;

    let mut root_node = DisplayNode {
        size: 0,
        name: PathBuf::from(zip_file.file_name().unwrap_or_default()),
        children: Vec::new(),
    };

    archive
        .for_each_entries(|entry, _reader| {
            let entry_path = Path::new(entry.name());

            let mut current_node = &mut root_node;
            for (i, component) in entry_path.components().enumerate() {
                if i >= depth {
                    break;
                }
                if let Some(component_str) = component.as_os_str().to_str() {
                    let child_name = PathBuf::from(component_str);
                    let child_node_index = current_node
                        .children
                        .iter()
                        .position(|n| n.name == child_name);
                    if let Some(index) = child_node_index {
                        current_node = &mut current_node.children[index];
                    } else {
                        let new_node = DisplayNode {
                            size: 0,
                            name: child_name,
                            children: Vec::new(),
                        };
                        current_node.children.push(new_node);
                        current_node = current_node.children.last_mut().unwrap();
                    }
                }
            }

            if !entry.is_directory() {
                current_node.size += if use_actual_size {
                    entry.size
                } else {
                    entry.compressed_size
                };
            }
            Ok(true)
        })
        .map_err(|e| {
            IoError::new(
                std::io::ErrorKind::Other,
                format!("Error processing 7z entries: {}", e),
            )
        })?;

    fn update_directory_sizes(node: &mut DisplayNode) {
        for child in &mut node.children {
            update_directory_sizes(child);
            node.size += child.size;
        }
    }

    update_directory_sizes(&mut root_node);

    fn sort_children(node: &mut DisplayNode) {
        node.children.sort_by(|a, b| b.size.cmp(&a.size));
        for child in &mut node.children {
            sort_children(child);
        }
    }

    sort_children(&mut root_node);

    Ok(root_node)
}
