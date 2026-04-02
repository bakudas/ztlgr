use anyhow::Result;
use std::path::PathBuf;

#[path = "storage/mod.rs"]
mod storage;

fn main() -> Result<()> {
    // Test storage module
    use storage::{MarkdownStorage, Storage, Vault, Format};
    
    // Create a test vault
    let vault_path = PathBuf::from("/tmp/test-vault");
    let vault = Vault::new(vault_path.clone(), Format::Markdown);
    
    // Initialize vault
    if !vault.exists() {
        vault.initialize()?;
        println!("Created vault at {:?}", vault_path);
    }
    
    // Test markdown storage
    let md_storage = MarkdownStorage::new();
    
    println!("Storage module test successful!");
    
    Ok(())
}