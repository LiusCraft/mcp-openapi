use crate::models::{ApiDefinition, ApiStatus, ApiStore};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// API 存储管理器
pub struct ApiStorageManager {
    /// 存储文件路径
    file_path: PathBuf,
    /// 内存中的 API 存储
    store: Arc<RwLock<ApiStore>>,
}

impl ApiStorageManager {
    /// 创建新的存储管理器
    pub async fn new(file_path: PathBuf) -> Result<Self> {
        let store = if file_path.exists() {
            let content = tokio::fs::read_to_string(&file_path)
                .await
                .context("Failed to read API store file")?;
            serde_json::from_str(&content).context("Failed to parse API store file")?
        } else {
            ApiStore::default()
        };

        Ok(Self {
            file_path,
            store: Arc::new(RwLock::new(store)),
        })
    }

    /// 保存到文件
    async fn save(&self) -> Result<()> {
        let store = self.store.read().await;
        let content = serde_json::to_string_pretty(&*store)?;

        // 确保父目录存在
        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.file_path, content)
            .await
            .context("Failed to write API store file")?;
        Ok(())
    }

    /// 获取所有 API
    pub async fn list_apis(&self) -> Vec<ApiDefinition> {
        let store = self.store.read().await;
        store.apis.clone()
    }

    /// 获取所有启用的 API
    pub async fn list_enabled_apis(&self) -> Vec<ApiDefinition> {
        let store = self.store.read().await;
        store
            .apis
            .iter()
            .filter(|api| api.status == ApiStatus::Enabled)
            .cloned()
            .collect()
    }

    /// 根据 ID 获取 API
    pub async fn get_api(&self, id: &str) -> Option<ApiDefinition> {
        let store = self.store.read().await;
        store.apis.iter().find(|api| api.id == id).cloned()
    }

    /// 根据名称获取 API
    pub async fn get_api_by_name(&self, name: &str) -> Option<ApiDefinition> {
        let store = self.store.read().await;
        store.apis.iter().find(|api| api.name == name).cloned()
    }

    /// 添加新 API
    pub async fn add_api(&self, api: ApiDefinition) -> Result<ApiDefinition> {
        {
            let mut store = self.store.write().await;

            // 检查名称是否重复
            if store.apis.iter().any(|a| a.name == api.name) {
                anyhow::bail!("API with name '{}' already exists", api.name);
            }

            store.apis.push(api.clone());
        }

        self.save().await?;
        Ok(api)
    }

    /// 更新 API
    pub async fn update_api(&self, id: &str, mut updated: ApiDefinition) -> Result<ApiDefinition> {
        {
            let mut store = self.store.write().await;

            let index = store
                .apis
                .iter()
                .position(|api| api.id == id)
                .context("API not found")?;

            // 检查名称是否与其他 API 重复
            if store
                .apis
                .iter()
                .enumerate()
                .any(|(i, a)| i != index && a.name == updated.name)
            {
                anyhow::bail!("API with name '{}' already exists", updated.name);
            }

            updated.id = id.to_string();
            updated.updated_at = chrono::Utc::now().to_rfc3339();
            store.apis[index] = updated.clone();
        }

        self.save().await?;
        Ok(updated)
    }

    /// 删除 API
    pub async fn delete_api(&self, id: &str) -> Result<ApiDefinition> {
        let removed = {
            let mut store = self.store.write().await;

            let index = store
                .apis
                .iter()
                .position(|api| api.id == id)
                .context("API not found")?;

            store.apis.remove(index)
        };

        self.save().await?;
        Ok(removed)
    }

    /// 启用 API
    pub async fn enable_api(&self, id: &str) -> Result<ApiDefinition> {
        let api = {
            let mut store = self.store.write().await;

            let api = store
                .apis
                .iter_mut()
                .find(|api| api.id == id)
                .context("API not found")?;

            api.status = ApiStatus::Enabled;
            api.updated_at = chrono::Utc::now().to_rfc3339();
            api.clone()
        };

        self.save().await?;
        Ok(api)
    }

    /// 禁用 API
    pub async fn disable_api(&self, id: &str) -> Result<ApiDefinition> {
        let api = {
            let mut store = self.store.write().await;

            let api = store
                .apis
                .iter_mut()
                .find(|api| api.id == id)
                .context("API not found")?;

            api.status = ApiStatus::Disabled;
            api.updated_at = chrono::Utc::now().to_rfc3339();
            api.clone()
        };

        self.save().await?;
        Ok(api)
    }

    /// 按标签筛选 API
    pub async fn list_apis_by_tag(&self, tag: &str) -> Vec<ApiDefinition> {
        let store = self.store.read().await;
        store
            .apis
            .iter()
            .filter(|api| api.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    // ========== 变量管理方法 ==========

    /// 获取所有变量
    pub async fn get_variables(&self) -> HashMap<String, String> {
        let store = self.store.read().await;
        store.variables.clone()
    }

    /// 获取单个变量
    pub async fn get_variable(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        store.variables.get(key).cloned()
    }

    /// 设置变量
    pub async fn set_variable(&self, key: String, value: String) -> Result<()> {
        {
            let mut store = self.store.write().await;
            store.variables.insert(key, value);
        }
        self.save().await
    }

    /// 删除变量
    pub async fn delete_variable(&self, key: &str) -> Result<bool> {
        let deleted = {
            let mut store = self.store.write().await;
            store.variables.remove(key).is_some()
        };
        if deleted {
            self.save().await?;
        }
        Ok(deleted)
    }

    /// 批量设置变量
    #[allow(dead_code)]
    pub async fn set_variables(&self, variables: HashMap<String, String>) -> Result<()> {
        {
            let mut store = self.store.write().await;
            for (key, value) in variables {
                store.variables.insert(key, value);
            }
        }
        self.save().await
    }
}
