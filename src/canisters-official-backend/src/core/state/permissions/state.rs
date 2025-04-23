// src/core/state/permissions/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::{HashMap};

    use ic_stable_structures::memory_manager::MemoryId;
    use ic_stable_structures::{StableBTreeMap, DefaultMemoryImpl, StableVec};

    use crate::core::state::permissions::types::{DirectoryPermissionIDList, SystemPermission, SystemPermissionID, SystemPermissionIDList, SystemResourceID};
    use crate::core::{
        state::permissions::types::{
            DirectoryPermission, DirectoryPermissionID, PermissionGranteeID
        },
    };
    use crate::rest::directory::types::DirectoryResourceID;
    use crate::MEMORY_MANAGER;

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;

    pub const DIR_PERMISSIONS_MEMORY_ID: MemoryId = MemoryId::new(44);
    pub const DIR_PERMISSIONS_BY_RESOURCE_MEMORY_ID: MemoryId = MemoryId::new(45);
    pub const DIR_GRANTEE_PERMISSIONS_MEMORY_ID: MemoryId = MemoryId::new(46);
    pub const DIR_PERMISSIONS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(47);
    
    pub const SYS_PERMISSIONS_MEMORY_ID: MemoryId = MemoryId::new(48);
    pub const SYS_PERMISSIONS_BY_RESOURCE_MEMORY_ID: MemoryId = MemoryId::new(49);
    pub const SYS_GRANTEE_PERMISSIONS_MEMORY_ID: MemoryId = MemoryId::new(50);
    pub const SYS_PERMISSIONS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(51);

    thread_local! {
        // Main storage for directory permissions
        pub(crate) static DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<DirectoryPermissionID, DirectoryPermission, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DIR_PERMISSIONS_MEMORY_ID))
            )
        );

        // Resource-based indices for O(1) lookups
        pub(crate) static DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE: RefCell<StableBTreeMap<DirectoryResourceID, DirectoryPermissionIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DIR_PERMISSIONS_BY_RESOURCE_MEMORY_ID))
            )
        );

        // Grantee-based indices for O(1) lookups
        pub(crate) static DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE: RefCell<StableBTreeMap<PermissionGranteeID, DirectoryPermissionIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DIR_GRANTEE_PERMISSIONS_MEMORY_ID))
            )
        );

        // Time-based indices
        pub(crate) static DIRECTORY_PERMISSIONS_BY_TIME_LIST: RefCell<DirectoryPermissionIDList> = RefCell::new(
            DirectoryPermissionIDList::new()
        );

        // Main storage for system permissions
        pub(crate) static SYSTEM_PERMISSIONS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<SystemPermissionID, SystemPermission, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(SYS_PERMISSIONS_MEMORY_ID))
            )
        );

        // Resource-based indices for O(1) lookups
        pub(crate) static SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE: RefCell<StableBTreeMap<SystemResourceID, SystemPermissionIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(SYS_PERMISSIONS_BY_RESOURCE_MEMORY_ID))
            )
        );

        // Grantee-based indices for O(1) lookups
        pub(crate) static SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE: RefCell<StableBTreeMap<PermissionGranteeID, SystemPermissionIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(SYS_GRANTEE_PERMISSIONS_MEMORY_ID))
            )
        );

        // Time-based indices
        pub(crate) static SYSTEM_PERMISSIONS_BY_TIME_LIST: RefCell<StableVec<SystemPermissionID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(SYS_PERMISSIONS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize SYSTEM_PERMISSIONS_BY_TIME_LIST")
        );
    }

    pub fn initialize() {
        // Force thread_locals in this module to initialize
        DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|_| {});
        DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|_| {});
        DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|_| {});
        DIRECTORY_PERMISSIONS_BY_TIME_LIST.with(|_| {});
        SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|_| {});
        SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|_| {});
        SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|_| {});
        SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|_| {});
    }

}



// Helper functions for managing permissions state
pub mod helpers {
    use super::state::*;
    use crate::core::state::permissions::types::{
        DirectoryPermission, DirectoryPermissionID, DirectoryPermissionIDList, PermissionGranteeID, SystemPermission, SystemPermissionID, SystemPermissionIDList
    };
    use crate::rest::directory::types::DirectoryResourceID;
    use crate::core::state::permissions::types::SystemResourceID;
    use crate::MEMORY_MANAGER;
    use ic_stable_structures::StableVec;

    // Directory permission helpers

    /// Removes a permission from the resource-indexed map
    pub fn remove_directory_permission_from_resource(
        resource_id: &DirectoryResourceID, 
        permission_id: &DirectoryPermissionID
    ) {
        DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            let mut permissions = permissions_by_resource.borrow_mut();
            if let Some(current_list) = permissions.get(resource_id) {
                let mut updated_list = DirectoryPermissionIDList::new();
                
                for i in 0..current_list.permissions.len() {
                    if let Some(id) = current_list.permissions.get(i) {
                        if id != permission_id {
                            updated_list.add(id.clone());
                        }
                    }
                }
                
                if updated_list.is_empty() {
                    permissions.remove(resource_id);
                } else {
                    permissions.insert(resource_id.clone(), updated_list);
                }
            }
        });
    }

    /// Removes a permission from the grantee-indexed map
    pub fn remove_directory_permission_from_grantee(
        grantee_id: &PermissionGranteeID,
        permission_id: &DirectoryPermissionID
    ) {
        DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            let mut permissions = grantee_permissions.borrow_mut();
            if let Some(current_list) = permissions.get(grantee_id) {
                let mut updated_list = DirectoryPermissionIDList::new();
                
                for i in 0..current_list.permissions.len() {
                    if let Some(id) = current_list.permissions.get(i) {
                        if id != permission_id {
                            updated_list.add(id.clone());
                        }
                    }
                }
                
                if updated_list.is_empty() {
                    permissions.remove(grantee_id);
                } else {
                    permissions.insert(grantee_id.clone(), updated_list);
                }
            }
        });
    }

    /// Adds a permission to the resource-indexed map
    pub fn add_directory_permission_to_resource(
        resource_id: &DirectoryResourceID,
        permission_id: &DirectoryPermissionID
    ) {
        DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            let mut table = permissions_by_resource.borrow_mut();
            
            let mut resource_list = match table.get(resource_id) {
                Some(existing_list) => {
                    let mut list_copy = DirectoryPermissionIDList::new();
                    for i in 0..existing_list.permissions.len() {
                        if let Some(id) = existing_list.permissions.get(i) {
                            list_copy.add(id.clone());
                        }
                    }
                    list_copy
                },
                None => DirectoryPermissionIDList::new()
            };
            
            resource_list.add(permission_id.clone());
            table.insert(resource_id.clone(), resource_list);
        });
    }

    /// Adds a permission to the grantee-indexed map
    pub fn add_directory_permission_to_grantee(
        grantee_id: &PermissionGranteeID,
        permission_id: &DirectoryPermissionID
    ) {
        DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            let mut table = grantee_permissions.borrow_mut();
            
            let mut grantee_list = match table.get(grantee_id) {
                Some(existing_list) => {
                    let mut list_copy = DirectoryPermissionIDList::new();
                    for i in 0..existing_list.permissions.len() {
                        if let Some(id) = existing_list.permissions.get(i) {
                            list_copy.add(id.clone());
                        }
                    }
                    list_copy
                },
                None => DirectoryPermissionIDList::new()
            };
            
            grantee_list.add(permission_id.clone());
            table.insert(grantee_id.clone(), grantee_list);
        });
    }

    /// Updates the time-indexed list of permissions
    pub fn update_directory_permissions_time_list(
        permission_id: &DirectoryPermissionID,
        add: bool
    ) {
        DIRECTORY_PERMISSIONS_BY_TIME_LIST.with(|permissions_by_time| {
            if add {
                // Add to the time list
                permissions_by_time.borrow_mut().add(permission_id.clone());
            } else {
                // Remove from the time list
                let mut new_list = DirectoryPermissionIDList::new();
                
                let list_ref = permissions_by_time.borrow();
                for i in 0..list_ref.permissions.len() {
                    if let Some(id) = list_ref.permissions.get(i) {
                        if id != permission_id {
                            new_list.add(id.clone());
                        }
                    }
                }
                
                drop(list_ref);
                *permissions_by_time.borrow_mut() = new_list;
            }
        });
    }

    /// Gets all permission IDs for a specific directory resource
    pub fn get_directory_permission_ids_for_resource(
        resource_id: &DirectoryResourceID
    ) -> Option<DirectoryPermissionIDList> {
        let mut result = None;
        
        DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            if let Some(permission_ids) = permissions_by_resource.borrow().get(resource_id) {
                result = Some(permission_ids.clone());
            }
        });
        
        result
    }

    /// Gets a directory permission by ID
    pub fn get_directory_permission_by_id(
        permission_id: &DirectoryPermissionID
    ) -> Option<DirectoryPermission> {
        let mut result = None;
        
        DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            if let Some(permission) = permissions.borrow().get(permission_id) {
                result = Some(permission.clone());
            }
        });
        
        result
    }

    // System permission helpers

    /// Removes a permission from the resource-indexed map
    pub fn remove_system_permission_from_resource(
        resource_id: &SystemResourceID, 
        permission_id: &SystemPermissionID
    ) {
        SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            let mut permissions = permissions_by_resource.borrow_mut();
            if let Some(current_list) = permissions.get(resource_id) {
                let mut updated_list = SystemPermissionIDList::new();
                
                for i in 0..current_list.permissions.len() {
                    if let Some(id) = current_list.permissions.get(i) {
                        if id != permission_id {
                            updated_list.add(id.clone());
                        }
                    }
                }
                
                if updated_list.is_empty() {
                    permissions.remove(resource_id);
                } else {
                    permissions.insert(resource_id.clone(), updated_list);
                }
            }
        });
    }

    /// Removes a permission from the grantee-indexed map
    pub fn remove_system_permission_from_grantee(
        grantee_id: &PermissionGranteeID,
        permission_id: &SystemPermissionID
    ) {
        SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            let mut permissions = grantee_permissions.borrow_mut();
            if let Some(current_list) = permissions.get(grantee_id) {
                let mut updated_list = SystemPermissionIDList::new();
                
                for i in 0..current_list.permissions.len() {
                    if let Some(id) = current_list.permissions.get(i) {
                        if id != permission_id {
                            updated_list.add(id.clone());
                        }
                    }
                }
                
                if updated_list.is_empty() {
                    permissions.remove(grantee_id);
                } else {
                    permissions.insert(grantee_id.clone(), updated_list);
                }
            }
        });
    }

    /// Adds a permission to the resource-indexed map
    pub fn add_system_permission_to_resource(
        resource_id: &SystemResourceID,
        permission_id: &SystemPermissionID
    ) {
        SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            let mut table = permissions_by_resource.borrow_mut();
            
            let mut resource_list = match table.get(resource_id) {
                Some(existing_list) => {
                    let mut list_copy = SystemPermissionIDList::new();
                    for i in 0..existing_list.permissions.len() {
                        if let Some(id) = existing_list.permissions.get(i) {
                            list_copy.add(id.clone());
                        }
                    }
                    list_copy
                },
                None => SystemPermissionIDList::new()
            };
            
            resource_list.add(permission_id.clone());
            table.insert(resource_id.clone(), resource_list);
        });
    }

    /// Adds a permission to the grantee-indexed map
    pub fn add_system_permission_to_grantee(
        grantee_id: &PermissionGranteeID,
        permission_id: &SystemPermissionID
    ) {
        SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            let mut table = grantee_permissions.borrow_mut();
            
            let mut grantee_list = match table.get(grantee_id) {
                Some(existing_list) => {
                    let mut list_copy = SystemPermissionIDList::new();
                    for i in 0..existing_list.permissions.len() {
                        if let Some(id) = existing_list.permissions.get(i) {
                            list_copy.add(id.clone());
                        }
                    }
                    list_copy
                },
                None => SystemPermissionIDList::new()
            };
            
            grantee_list.add(permission_id.clone());
            table.insert(grantee_id.clone(), grantee_list);
        });
    }

    /// Updates the time-indexed list of permissions
    pub fn update_system_permissions_time_list(
        permission_id: &SystemPermissionID,
        add: bool
    ) {
        SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|permissions_by_time| {
            let mut list = permissions_by_time.borrow_mut();
            
            if add {
                // Add to the time list
                list.push(permission_id)
                    .expect("Failed to add permission to time list");
            } else {
                // Remove from the time list - we need to rebuild it without the removed item
                let mut new_list = StableVec::init(
                    MEMORY_MANAGER.with(|m| m.borrow().get(SYS_PERMISSIONS_BY_TIME_MEMORY_ID))
                ).expect("Failed to initialize new time list");
                
                // Copy all items except the one to remove
                for i in 0..list.len() {
                    if let Some(id) = list.get(i) {
                        if id != *permission_id {
                            new_list.push(&id)
                                .expect("Failed to add permission to new time list");
                        }
                    }
                }
                
                // Replace the old list with the new one
                *list = new_list;
            }
        });
    }


    /// Gets all permission IDs for a specific resource
    pub fn get_system_permission_ids_for_resource(
        resource_id: &SystemResourceID
    ) -> Option<SystemPermissionIDList> {
        let mut result = None;
        
        SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            if let Some(permission_ids) = permissions_by_resource.borrow().get(resource_id) {
                result = Some(permission_ids.clone());
            }
        });
        
        result
    }

    /// Gets all permission IDs from the time-ordered list
    pub fn get_system_permissions_by_time() -> SystemPermissionIDList {
        let mut result = SystemPermissionIDList::new();
        
        SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|time_list| {
            let list = time_list.borrow();
            for i in 0..list.len() {
                if let Some(id) = list.get(i) {
                    result.add(id);
                }
            }
        });
        
        result
    }

    /// Gets a system permission by ID
    pub fn get_system_permission_by_id(
        permission_id: &SystemPermissionID
    ) -> Option<SystemPermission> {
        let mut result = None;
        
        SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            if let Some(permission) = permissions.borrow().get(permission_id) {
                result = Some(permission.clone());
            }
        });
        
        result
    }
}