#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![warn(missing_docs, future_incompatible, keyword_idents)]

pub mod device;
pub mod error;
pub mod filter;
pub mod frame;
pub mod pipeline;
pub mod stream;
pub(crate) mod sys;

use std::sync::{Arc, Mutex, Weak};
use std::sync::OnceLock;

use sys::context::OBContext;

#[doc(inline)]
pub use crate::sys::enums::OBDeviceType as DeviceType;

#[doc(inline)]
pub use crate::sys::enums::OBFormat as Format;

#[doc(inline)]
pub use crate::sys::enums::OBSensorType as SensorType;

#[doc(inline)]
pub use crate::sys::enums::OBStreamType as StreamType;

#[doc(inline)]
pub use crate::sys::enums::OBPermissionType as PermissionType;

#[doc(inline)]
pub use crate::sys::enums::OBHoleFillMode as HoleFillMode;

#[doc(inline)]
pub use crate::sys::enums::OBConvertFormat as ConvertType;

#[doc(inline)]
pub use crate::sys::enums::OBCoordinateSystem as CoordinateSystem;

#[doc(inline)]
pub use crate::sys::enums::OBLogSeverity as LogSeverity;

/// Set the logger severity level for the Orbbec SDK
pub use crate::sys::set_logger_severity;

/// Global context singleton similar to the C++ SDK implementation.
/// 
/// IMPORTANT:
/// 1. Do NOT store the Arc<Context> for long-term use.
/// 2. Always acquire the instance when needed and release it immediately.
/// 3. For long-term storage (e.g., in member variables), use Weak<Context> instead
///    of Arc<Context> to avoid potential lifetime and ownership issues.
static GLOBAL_CONTEXT: OnceLock<Mutex<Weak<Context>>> = OnceLock::new();

/// Context Manager - Singleton pattern matching the C++ SDK
/// 
/// The Orbbec SDK uses a global singleton Context instance internally.
/// This implementation provides thread-safe access to that singleton while
/// following the same patterns as the C++ API.
#[derive(Clone)]
pub struct Context {
    inner: Arc<OBContext>,
}

impl Context {
    /// Returns a shared reference to the global Context instance.
    /// 
    /// ### Arguments
    /// * `config_path` - Config file path. Used only on the first call when the Context is created.
    /// 
    /// ### Important Notes:
    /// 1. Do NOT store this Arc<Context> for long-term use.
    /// 2. Always acquire the instance when needed and release it immediately.
    /// 3. If a long-term reference is required (e.g., in a member variable), store a 
    ///    `std::sync::Weak<Context>` instead to avoid potential lifetime and ownership issues.
    pub fn global_instance(config_path: Option<&str>) -> Result<Arc<Self>, error::OrbbecError> {
        let global_mutex = GLOBAL_CONTEXT.get_or_init(|| Mutex::new(Weak::new()));
        let mut global_weak = global_mutex.lock().unwrap();
        
        // Try to upgrade existing weak reference
        if let Some(existing) = global_weak.upgrade() {
            return Ok(existing);
        }
        
        // Create new context instance
        let ob_context = if let Some(_path) = config_path {
            // TODO: Add config path support to OBContext::new()
            // For now, use default config
            OBContext::new().map_err(error::OrbbecError::from)?
        } else {
            OBContext::new().map_err(error::OrbbecError::from)?
        };
        
        let context = Arc::new(Context {
            inner: Arc::new(ob_context),
        });
        
        // Store weak reference
        *global_weak = Arc::downgrade(&context);
        
        Ok(context)
    }
    
    /// Create a new context (legacy method for backward compatibility)
    /// 
    /// ### Deprecated
    /// Consider using `Context::global_instance()` instead for proper singleton behavior
    /// matching the C++ SDK.
    pub fn new() -> Result<Self, error::OrbbecError> {
        let ctx = OBContext::new().map_err(error::OrbbecError::from)?;

        Ok(Context { 
            inner: Arc::new(ctx),
        })
    }

    /// Query the list of connected devices
    pub fn query_device_list<'a>(&'a self) -> Result<device::DeviceList<'a>, error::OrbbecError> {
        let list = self
            .inner
            .query_device_list()
            .map_err(error::OrbbecError::from)?;

        Ok(device::DeviceList::new(list, self))
    }

    /// Enable or disable network device enumeration
    pub fn enable_net_device_enumeration(&self, enable: bool) -> Result<(), error::OrbbecError> {
        self.inner
            .enable_net_device_enumeration(enable)
            .map_err(error::OrbbecError::from)
    }

    /// Create a weak reference to this context for long-term storage
    /// 
    /// Use this when you need to store a context reference in a struct or
    /// for long-term use to avoid ownership issues, as recommended by Orbbec.
    pub fn weak_ref(context: &Arc<Context>) -> Weak<Context> {
        Arc::downgrade(context)
    }
}
