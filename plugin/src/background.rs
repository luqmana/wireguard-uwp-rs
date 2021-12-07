//! The entrypoint for the background task where our actual VPN plugin runs.

use windows::{
    self as Windows,
    core::*,
    ApplicationModel::Background::IBackgroundTaskInstance,
    Win32::Foundation::{E_INVALIDARG, E_NOINTERFACE, S_OK},
    Win32::System::WinRT::IActivationFactory,
};

/// The WinRT Activatable Class which acts as the entrypoint for the background tasks
/// which get invoked to handle the actual VPN tunnel.
#[implement(Windows::ApplicationModel::Background::IBackgroundTask)]
pub struct VpnBackgroundTask;

impl VpnBackgroundTask {
    fn Run(&self, _task: &Option<IBackgroundTaskInstance>) -> Result<()> {
        todo!()
    }
}

/// A factory object to generate `VpnBackgroundTask`.
///
/// Returned by `DllGetActivationFactory` when the system attempts to get an
/// instance of `VpnBackgroundTask`.
#[implement(Windows::Win32::System::WinRT::IActivationFactory)]
struct VpnBackgroundTaskFactory;

impl VpnBackgroundTaskFactory {
    /// Creates and returns a new instance of `VpnBackgroundTask`.
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Ok(VpnBackgroundTask.into())
    }
}

/// Called by any consumers of this library attempting to get instances of any activatable
/// Windows Runtime classes we support.
///
/// When the system is ready to launch our VPN background task, it needs to get a reference
/// to our `VpnBackgroundTask` object. It can do so because as part of our `AppxManifest.xml`
/// we list out which Activatable Classes (VpnBackgroundTask) we want registered during App
/// installation. Furthermore, we specify that the component is hosted in our DLL. From there,
/// it knows to query us via the `DllGetActivationFactory` function we export to return some
/// object implementing `IActivationFactory` which knows how to create new instances of the
/// target WinRT runtime class.
#[no_mangle]
pub unsafe extern "system" fn DllGetActivationFactory(
    activatableClassId: HSTRING,
    factory: *mut Option<IActivationFactory>,
) -> HRESULT {
    if activatableClassId.is_empty() || factory.is_null() {
        return E_INVALIDARG;
    }

    // Return the appropriate factory based on which class was requested
    if activatableClassId == "VpnBackgroundTask" {
        *factory = Some(VpnBackgroundTaskFactory.into());
    } else {
        // Otherwise bail for any class we don't support
        *factory = None;
        return E_NOINTERFACE;
    }

    S_OK
}