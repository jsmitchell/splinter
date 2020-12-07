// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Implementation of a `StoreFactory` for in memory

#[cfg(feature = "sqlite")]
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};

#[cfg(feature = "biome-oauth")]
use crate::biome::MemoryOAuthUserStore;
#[cfg(feature = "biome-credentials")]
use crate::biome::{
    CredentialsStore, MemoryCredentialsStore, MemoryRefreshTokenStore, RefreshTokenStore,
};
#[cfg(feature = "biome-key-management")]
use crate::biome::{KeyStore, MemoryKeyStore};
#[cfg(feature = "biome")]
use crate::biome::{MemoryUserStore, UserStore};

use super::StoreFactory;

/// A `StoryFactory` backed by memory.
#[derive(Default)]
pub struct MemoryStoreFactory {
    #[cfg(feature = "biome-credentials")]
    biome_credentials_store: MemoryCredentialsStore,
    #[cfg(feature = "biome-key-management")]
    biome_key_store: MemoryKeyStore,
    #[cfg(feature = "biome-credentials")]
    biome_refresh_token_store: MemoryRefreshTokenStore,
    #[cfg(feature = "biome")]
    biome_user_store: MemoryUserStore,
    #[cfg(feature = "biome-oauth")]
    biome_oauth_user_store: MemoryOAuthUserStore,
}

impl MemoryStoreFactory {
    pub fn new() -> Self {
        #[cfg(feature = "biome-credentials")]
        let biome_credentials_store = MemoryCredentialsStore::new();

        #[cfg(all(feature = "biome-key-management", feature = "biome-credentials"))]
        let biome_key_store = MemoryKeyStore::new(biome_credentials_store.clone());
        #[cfg(all(feature = "biome-key-management", not(feature = "biome-credentials")))]
        let biome_key_store = MemoryKeyStore::new();

        #[cfg(feature = "biome-credentials")]
        let biome_user_store = MemoryUserStore::new(biome_credentials_store.clone());

        #[cfg(all(not(feature = "biome-credentials"), features = "biome"))]
        let biome_user_store = MemoryUserStore::new();

        #[cfg(feature = "biome-oauth")]
        let biome_oauth_user_store = MemoryOAuthUserStore::new();

        Self {
            #[cfg(feature = "biome-credentials")]
            biome_credentials_store,
            #[cfg(feature = "biome-key-management")]
            biome_key_store,
            #[cfg(feature = "biome-credentials")]
            biome_refresh_token_store: MemoryRefreshTokenStore::new(),
            #[cfg(feature = "biome")]
            biome_user_store,
            #[cfg(feature = "biome-oauth")]
            biome_oauth_user_store,
        }
    }
}

impl StoreFactory for MemoryStoreFactory {
    #[cfg(feature = "biome-credentials")]
    fn get_biome_credentials_store(&self) -> Box<dyn CredentialsStore> {
        Box::new(self.biome_credentials_store.clone())
    }

    #[cfg(feature = "biome-key-management")]
    fn get_biome_key_store(&self) -> Box<dyn KeyStore> {
        Box::new(self.biome_key_store.clone())
    }

    #[cfg(feature = "biome-credentials")]
    fn get_biome_refresh_token_store(&self) -> Box<dyn RefreshTokenStore> {
        Box::new(self.biome_refresh_token_store.clone())
    }

    #[cfg(feature = "biome")]
    fn get_biome_user_store(&self) -> Box<dyn UserStore> {
        Box::new(self.biome_user_store.clone())
    }

    #[cfg(feature = "biome-oauth")]
    fn get_biome_oauth_user_store(&self) -> Box<dyn crate::biome::OAuthUserStore> {
        Box::new(self.biome_oauth_user_store.clone())
    }

    #[cfg(all(feature = "admin-service", feature = "sqlite"))]
    fn get_admin_service_store(&self) -> Box<dyn crate::admin::store::AdminServiceStore> {
        let connection_manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder()
            .max_size(1)
            .build(connection_manager)
            .expect("Failed to build connection pool");

        crate::migrations::run_sqlite_migrations(
            &*pool.get().expect("Failed to get connection for migrations"),
        )
        .expect("Failed to run migrations");

        Box::new(crate::admin::store::diesel::DieselAdminServiceStore::new(
            pool,
        ))
    }

    #[cfg(all(feature = "admin-service", not(feature = "sqlite")))]
    fn get_admin_service_store(&self) -> Box<dyn crate::admin::store::AdminServiceStore> {
        unimplemented!()
    }
}
