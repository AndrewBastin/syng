use anyhow::{bail, Result};
use std::{collections::{HashMap, BTreeMap}, sync::RwLock, time::SystemTime};

use actix_web::{get, middleware::Logger, web, App, HttpServer, Responder, post};
use syng::{
    backend::SyngBackend, delta::{generate_delta_from_point, SyngDelta, apply_delta}, objects::SyngObjectDef,
    tree_ops::get_descendent_objects,
};
use syng_demo_common::backend::{
    BackendCurrRootResult, BackendFullPullResult, BackendPullFromResult, BackendPullFromError,
    BackendPushResult, BackendPushError
};

struct DataBackend {
    objects: HashMap<String, SyngObjectDef>,
    root_object_id: Option<String>
}

impl Default for DataBackend {
    fn default() -> Self {
        let root_obj = SyngObjectDef {
            fields: BTreeMap::new(),
            children: vec![],
        };

        let root_hash = root_obj.get_hash().unwrap();

        let objects = HashMap::from([
            (root_hash.clone(), root_obj)
        ]);

        Self {
            objects,
            root_object_id: Some(root_hash)
        }
    }
}

struct BackendState {
    data: RwLock<DataBackend>
}

impl SyngBackend for DataBackend {
    fn get_root_object_id(&self) -> Option<String> {
        return self.root_object_id.clone();
    }

    fn get_root_object(&self) -> Option<SyngObjectDef> {
        let root_id = self.get_root_object_id()?;

        Some(
            self.objects
                .get(&root_id)
                .expect("Root ID is set to a value not in the object list")
                .clone(),
        )
    }

    fn set_root_object(&mut self, node_id: &str) -> Result<()> {
        if !self.has_object(node_id) {
            bail!("INVALID_OBJ_ID");
        }

        self.root_object_id = Some(node_id.to_owned());

        println!("Root Object set to {:?}", self.root_object_id);

        Ok(())
    }

    fn read_object(&self, id: &str) -> Option<SyngObjectDef> {
        println!("Object Read: {}", id);

        Some(self.objects.get(id)?.clone())
    }

    fn write_object(&mut self, def: &SyngObjectDef) -> Result<String> {
        let hash = def.get_hash()?;
        println!("Object Write: [Hash: {}] {:?}", hash, def);

        self.objects.insert(hash.clone(), def.clone());

        Ok(hash)
    }
}

impl DataBackend {
    fn get_accesible_objects(&self) -> Option<Vec<SyngObjectDef>> {

        Some(match &self.root_object_id {
            None => vec![],
            Some(id) => get_descendent_objects(self, id)?,
        })
    }
}

#[get("/curr_root")]
async fn curr_root(state: web::Data<BackendState>) -> impl Responder {
    let result = state.data.read().unwrap().get_root_object_id();

    web::Json(BackendCurrRootResult { data: result })
}

#[get("/pull")]
async fn pull(state: web::Data<BackendState>) -> Option<impl Responder> {
    let backend = state.data.read().unwrap();

    let time_start = SystemTime::now();

    let root_id = backend.get_root_object_id();
    let accessible_objects = backend.get_accesible_objects()?;

    let time_end = SystemTime::now();
    let duration = time_end.duration_since(time_start).unwrap().as_millis();

    println!("Full pull took {}ms", duration);

    Some(web::Json(BackendFullPullResult {
        root_obj_id: root_id,
        objects: accessible_objects,
    }))
}

#[get("/pull_from/{hash}")]
async fn pull_from(hash: web::Path<String>, state: web::Data<BackendState>) -> impl Responder {
    let backend = state.data.read().unwrap();

    let time_start = SystemTime::now();

    if backend.get_root_object_id().is_none() {
        return web::Json(BackendPullFromResult {
            data: Err(BackendPullFromError::BackendHasNoRoot),
        });
    } else if !backend.has_object(&hash) {
        return web::Json(BackendPullFromResult {
            data: Err(BackendPullFromError::InvalidFromPoint),
        });
    }

    let time_end = SystemTime::now();
    let duration = time_end.duration_since(time_start).unwrap().as_millis();

    println!("Pull from {} took {}ms", hash, duration);

    let Some(delta) = generate_delta_from_point(&*backend, &hash) else { 
        return web::Json(BackendPullFromResult { 
            data: Err(BackendPullFromError::DeltaGenError)
        })
    };

    web::Json(BackendPullFromResult {
        data: Ok(delta)
    })
}

#[post("/push")]
async fn push(delta: web::Json<SyngDelta>, state: web::Data<BackendState>) -> impl Responder {
    let mut backend = state.data.write().unwrap();

    let delta = apply_delta(&mut *backend, &delta);

    web::Json(
        BackendPushResult {
            data: match delta {
                Err(e) => Err(BackendPushError::DeltaApplyFailed(e)),
                Ok(_) => Ok(())
            }
        }
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(BackendState {
        data: RwLock::new(DataBackend::default())
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .service(curr_root)
            .service(pull)
            .service(pull_from)
            .service(push)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
