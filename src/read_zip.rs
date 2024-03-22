use du_dust::DisplayNode;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub fn read_zip(
    zip_file: &PathBuf,
    depth: usize,
    use_actual_size: bool,
) -> Result<DisplayNode, std::io::Error> {
    let file = File::open(zip_file)?;
    let mut archive = ZipArchive::new(BufReader::new(file))?;

    let mut root_node = DisplayNode {
        size: 0,
        name: PathBuf::from(zip_file.file_name().unwrap_or_default()),
        children: Vec::new(),
    };

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
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

        current_node.size += if use_actual_size {
            entry.size()
        } else {
            entry.compressed_size()
        };
    }

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
