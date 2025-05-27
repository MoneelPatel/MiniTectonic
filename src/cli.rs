use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs::File;
use std::io::{self, Read};

use crate::{
    BlobId, TenantId,
    coordinator::Coordinator,
    Result,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the storage directory
    #[arg(short, long, default_value = "storage")]
    storage_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new tenant
    RegisterTenant {
        /// Tenant ID
        #[arg(short, long)]
        tenant: String,
    },

    /// List all registered tenants
    ListTenants,

    /// Store a blob
    Put {
        /// Tenant ID
        #[arg(short, long)]
        tenant: String,

        /// Path to the file to store
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Retrieve a blob
    Get {
        /// Tenant ID
        #[arg(short, long)]
        tenant: String,

        /// Blob ID
        #[arg(short, long)]
        blob: String,

        /// Output file path (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List blobs for a tenant
    ListBlobs {
        /// Tenant ID
        #[arg(short, long)]
        tenant: String,
    },

    /// Delete a blob
    Delete {
        /// Tenant ID
        #[arg(short, long)]
        tenant: String,

        /// Blob ID
        #[arg(short, long)]
        blob: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let coordinator = Coordinator::new(&cli.storage_dir)?;

    match &cli.command {
        Commands::RegisterTenant { tenant } => {
            coordinator.register_tenant(TenantId::new(tenant))?;
            println!("Tenant '{}' registered successfully", tenant);
        }

        Commands::ListTenants => {
            let tenants = coordinator.list_tenants()?;
            println!("Registered tenants:");
            for tenant in tenants {
                println!("- {}", tenant.as_str());
            }
        }

        Commands::Put { tenant, file } => {
            let tenant_id = TenantId::new(tenant);
            let file = File::open(file)?;
            let blob_id = coordinator.put_blob(&tenant_id, file)?;
            println!("Blob stored successfully. ID: {}", blob_id.to_string());
        }

        Commands::Get { tenant, blob, output } => {
            let tenant_id = TenantId::new(tenant);
            let blob_id = BlobId::from_str(blob)?;
            let mut reader = coordinator.get_blob(&tenant_id, &blob_id)?;

            match output {
                Some(path) => {
                    let mut file = File::create(path)?;
                    io::copy(&mut reader, &mut file)?;
                }
                None => {
                    let mut stdout = io::stdout();
                    io::copy(&mut reader, &mut stdout)?;
                }
            }
        }

        Commands::ListBlobs { tenant } => {
            let tenant_id = TenantId::new(tenant);
            let blobs = coordinator.list_blobs(&tenant_id)?;
            println!("Blobs for tenant '{}':", tenant);
            for metadata in blobs {
                println!("- ID: {}", metadata.blob_id.to_string());
                println!("  Size: {} bytes", metadata.size);
                println!("  Checksum: {}", metadata.checksum);
                println!("  Created: {}", metadata.created_at);
            }
        }

        Commands::Delete { tenant, blob } => {
            let tenant_id = TenantId::new(tenant);
            let blob_id = BlobId::from_str(blob)?;
            coordinator.delete_blob(&tenant_id, &blob_id)?;
            println!("Blob deleted successfully");
        }
    }

    Ok(())
}

impl BlobId {
    fn from_str(s: &str) -> Result<Self> {
        let uuid = uuid::Uuid::parse_str(s).map_err(|_| {
            crate::error::Error::System("Invalid blob ID format".into())
        })?;
        Ok(Self(uuid))
    }
} 