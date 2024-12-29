use std::sync::Arc;

use crate::vulkan::mappings::{cobra::IDInfo, CobraVulkan};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ResourceType {
    Sampler,
    Image
}

pub struct ResourceHandle {
    ty: ResourceType,
    pub id: u32,

    cobra: Arc<CobraVulkan>
}

impl ResourceHandle {
    pub fn new(cobra: Arc<CobraVulkan>, ty: ResourceType) -> ResourceHandle {
        // create key if doesnt exist (first time only)
        let mut id_infos = cobra.id_infos.lock().unwrap();
        if !id_infos.contains_key(&ty) {
            id_infos.insert(ty, IDInfo { id_counter: 0, recycled_ids: Vec::new() });
        }

        let id = if id_infos[&ty].recycled_ids.is_empty() {
            let id = id_infos[&ty].id_counter;
            id_infos.get_mut(&ty).unwrap().id_counter += 1;
            id
        } else {
            id_infos.get_mut(&ty).unwrap().recycled_ids.pop().unwrap()
        };

        drop(id_infos);
        ResourceHandle {
            ty, id, cobra
        }
    }
}

impl Drop for ResourceHandle {
    fn drop(&mut self) {
        self.cobra.id_infos.lock().unwrap().get_mut(&self.ty).unwrap().recycled_ids.push(self.id);
    }
}