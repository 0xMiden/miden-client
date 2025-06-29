use super::errors::SqliteStoreError;
use crate::store::StoreError;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use async_lock::Mutex;
use core::ffi::CStr;
use js_sys::Array;
use miden_objects::utils::{
    ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable,
};
use rusqlite::{Connection, vtab::array};
use sqlite_wasm_rs::{self as ffi, sahpool_vfs::install as install_opfs_vfs};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, MessageEvent, Url, Worker, WorkerOptions, WorkerType};

/// Message types for communication with the worker
pub enum WorkerRequest {
    Connect { path: String },
    Execute {},
}

impl Serializable for WorkerRequest {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        match self {
            Self::Connect { path } => {
                target.write_u8(0);
                path.write_into(target);
            },
            Self::Execute {} => {
                target.write_u8(1);
            },
        }
    }
}

impl Deserializable for WorkerRequest {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        match source.read_u8()? {
            0 => Ok(Self::Connect { path: String::read_from(source)? }),
            1 => Ok(Self::Execute {}),
            val => Err(DeserializationError::InvalidValue(format!("Invalid tag source: {val}"))),
        }
    }
}

pub struct WorkerResponse {
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
}

impl Serializable for WorkerResponse {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_bool(self.success);
        self.data.write_into(target);
        self.error.write_into(target);
    }
}

impl Deserializable for WorkerResponse {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let success = source.read_bool()?;
        let data = Option::<String>::read_from(source)?;
        let error = Option::<String>::read_from(source)?;
        Ok(Self { success, data, error })
    }
}

// TODO(Maks) - this naive pool implementation with interior mutability only for POC!!
// Consider to implement web workers based pooling
// E.g. https://github.com/w3reality/wasm-mt, https://github.com/paberr/wasmworker
// see also crates/web-client/js/index.js
// Worker lifetime and re-connects also have to be handled
pub struct SqlitePool {
    worker: Arc<Mutex<web_sys::Worker>>,
}

unsafe impl Send for SqlitePool {}
unsafe impl Sync for SqlitePool {}

impl SqlitePool {
    // TODO(Maks) initialize a worker from code?
    pub async fn connect(path: String) -> Result<Self, SqliteStoreError> {
        // let blob_options = BlobPropertyBag::new();
        // blob_options.set_type("application/javascript");

        // let code = Array::new();
        // code.push(&JsValue::from_str(WORKER_SCRIPT));

        // let script_url = Url::create_object_url_with_blob(
        //     &Blob::new_with_blob_sequence_and_options(&code.into(), &blob_options).map_err(
        //         |e| SqliteStoreError::DatabaseError(format!("failed to create worker blob: {e:?}")),
        //     )?,
        // )
        // .map_err(|e| {
        //     SqliteStoreError::DatabaseError(format!("failed to create worker blob url: {e:?}"))
        // })?;

        // let worker_options = WorkerOptions::new();
        // worker_options.set_type(WorkerType::Module);
        // let worker = Worker::new_with_options(&script_url, &worker_options).map_err(|e| {
        //     SqliteStoreError::ConfigurationError(format!("failed to create worker: {e:?}"))
        // })?;
        let worker_options = WorkerOptions::new();
        worker_options.set_type(WorkerType::Module);
        let worker = Worker::new_with_options(&"./workers/web-client-methods-worker.js", &worker_options).map_err(|e| {
            SqliteStoreError::ConfigurationError(format!("failed to create worker: {e:?}"))
        })?;

        let pool = Self { worker: Arc::new(Mutex::new(worker)) };

        let connect_msg = WorkerRequest::Connect { path };

        pool.send(connect_msg).await?;

        Ok(pool)
    }

    // TODO(Maks) think on errors returned
    pub async fn interact<F, R>(&self, f: F) -> Result<R, StoreError>
    where
        F: FnOnce(&mut Connection) -> Result<R, StoreError> + Send + 'static,
        R: Send + 'static + Deserializable,
    {
        let execute_req = WorkerRequest::Execute {};

        // TODO(Maks) add timeout
        let response = self
            .send(execute_req)
            .await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        if response.success {
            if let Some(data) = response.data {
                let result = R::read_from_bytes(data.as_bytes()).map_err(|e| {
                    StoreError::DatabaseError(format!("Failed to deserialize response: {}", e))
                })?;
                Ok(result)
            } else {
                Err(StoreError::DatabaseError("No data returned".to_string()))
            }
        } else {
            Err(StoreError::DatabaseError(
                response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    async fn send(&self, message: WorkerRequest) -> Result<WorkerResponse, SqliteStoreError> {
        let (promise, resolve, reject) = Self::create_promise();

        /// Create a closure to act on the message returned by the worker
        /// https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-web-worker.html
        fn get_on_msg_callback() -> Closure<dyn FnMut(MessageEvent)> {
            Closure::new(move |event: MessageEvent| {
                web_sys::console::log_2(&"Received response: ".into(), &event.data());

                // if let Ok(response_data) = event.data().dyn_into::<js_sys::Object>() {
                //     if let Ok(response_bytes) = serde_wasm_bindgen::from_value(response_data.into()) {
                //         let response = WorkerResponse::read_from_bytes(&response_bytes)
                //         resolve.call1(&JsValue::NULL, &serde_wasm_bindgen::to_value(&response).unwrap()).unwrap();
                //     } else {
                //         reject.call1(&JsValue::NULL, &JsValue::from_str("Failed to parse response")).unwrap();
                //     }
                // }
            })
        }

        let onmessage_callback = get_on_msg_callback();

        // // Set up message listener
        // let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        //     if let Ok(response_data) = event.data().dyn_into::<js_sys::Object>() {
        //         if let Ok(response) = serde_wasm_bindgen::from_value::<WorkerResponse>(response_data.into()) {
        //             resolve.call1(&JsValue::NULL, &serde_wasm_bindgen::to_value(&response).unwrap()).unwrap();
        //         } else {
        //             reject.call1(&JsValue::NULL, &JsValue::from_str("Failed to parse response")).unwrap();
        //         }
        //     }
        // }) as Box<dyn FnMut(_)>);
        // Send message to worker
        let message_bytes = message.to_bytes();
        let message_value = serde_wasm_bindgen::to_value(&message_bytes).map_err(|e| {
            SqliteStoreError::DatabaseError(format!("Failed to serialize message: {e:?}"))
        })?;
        {
            let worker = self.worker.lock().await;
            worker.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget(); // Keep the closure alive - TODO(Maks) check on memory leaks

            worker.post_message(&message_value).map_err(|e| {
                SqliteStoreError::ConfigurationError(format!("Failed to send message: {e:?}"))
            })?;
        }

        let response_value = JsFuture::from(promise).await.map_err(|e| {
            SqliteStoreError::DatabaseError(format!("Worker communication failed: {e:?}"))
        })?;

        let response_value_bytes: Vec<u8> = serde_wasm_bindgen::from_value(response_value)
            .map_err(|e| {
                SqliteStoreError::DatabaseError(format!("error parsing worker response: {e:?}"))
            })?;
        let response = WorkerResponse::read_from_bytes(&response_value_bytes).map_err(|e| {
            SqliteStoreError::DatabaseError(format!("Failed to deserialize response: {e:?}"))
        })?;

        Ok(response)
    }

    fn create_promise() -> (js_sys::Promise, js_sys::Function, js_sys::Function) {
        let mut resolve: Option<js_sys::Function> = None;
        let mut reject: Option<js_sys::Function> = None;

        let promise = js_sys::Promise::new(&mut |resolve_fn, reject_fn| {
            resolve = Some(resolve_fn);
            reject = Some(reject_fn);
        });

        (promise, resolve.unwrap(), reject.unwrap())
    }
}

const WORKER_SCRIPT: &str = r#"
    console.debug('Initializing worker');
    import init, { SqliteWorker } from './dist/wasm.js';

    let worker = null;

    async function initWorker() {
        await init();
        worker = new SqliteWorker();
    }

    self.onmessage = async function(event) {
        if (!worker) {
            await initWorker();
        }
        
        try {
            const response = await worker.handle_request(event.data);
            self.postMessage(response);
        } catch (error) {
            self.postMessage({
                success: false,
                data: null,
                error: error.toString()
            });
        }
    };
"#;

#[wasm_bindgen]
pub struct SqliteWorker {
    connection: Option<Connection>,
}

#[wasm_bindgen]
impl SqliteWorker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { connection: None }
    }

    #[wasm_bindgen]
    pub async fn handle_request(&mut self, message: JsValue) -> Result<JsValue, JsValue> {
        let message_bytes: Vec<u8> = serde_wasm_bindgen::from_value(message)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse message: {}", e)))?;
        let message: WorkerRequest = WorkerRequest::read_from_bytes(&message_bytes)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize message: {}", e)))?;

        match message {
            WorkerRequest::Connect { path } => self.connect(path).await,
            WorkerRequest::Execute {} => self.execute().await,
        }
    }

    async fn connect(&mut self, path: String) -> Result<JsValue, JsValue> {
        // TODO(Maks) check for proper lifetimes at FFI bounds (check sqlite3_open_v2)
        let mut path_bytes = path.into_bytes();
        path_bytes.push(b'\0');
        let cstr = CStr::from_bytes_with_nul(&path_bytes).map_err(|_| {
            JsValue::from_str("sqlite db name should be null terminated")
        })?;

        // This implementation of VFS is only available in a dedicated worker
        install_opfs_vfs(None, true)
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to install OPFS VFS: {}", e)))?;

        let mut db = core::ptr::null_mut();
        // TODO(Maks) justify unsafe invariants
        let ret = unsafe {
            ffi::sqlite3_open_v2(
                cstr.as_ptr().cast(),
                &mut db as *mut _,
                ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE,
                core::ptr::null(),
            )
        };

        if ret != ffi::SQLITE_OK {
            return Err(JsValue::from_str(&format!("Error opening SQLite DB: {ret}")));
        }

        // TODO(Maks) justify unsafe invariants
        let connection = unsafe { Connection::from_handle_owned(db) }
            .map_err(|e| JsValue::from_str(&format!("Failed to create connection: {e:?}")))?;

        // Feature used to support IN and NOT IN queries. We need to load
        // this module for every connection we create to the DB to support the
        // queries we want to run
        array::load_module(&connection)
            .map_err(|e| JsValue::from_str(&format!("error loading array module: {e:#?}")))?;

        // Enable foreign key checks.
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| JsValue::from_str(&format!("error enabling foreign keys: {e:#?}")))?;

        self.connection = Some(connection);

        let response = WorkerResponse { success: true, data: None, error: None };
        let reponse_bytes = response.to_bytes();
        Ok(serde_wasm_bindgen::to_value(&reponse_bytes).unwrap())
    }

    async fn execute(&mut self) -> Result<JsValue, JsValue> {
        let connection = self
            .connection
            .as_mut()
            .ok_or_else(|| JsValue::from_str("No database connection"))?;

        let response = WorkerResponse {
            success: true,
            data: Some("{}".to_string()),
            error: None,
        };
        let response_bytes = response.to_bytes();
        Ok(serde_wasm_bindgen::to_value(&response_bytes)
            .map_err(|_| JsValue::from_str("Serialization error"))?)
    }
}
