//! The entrypoint for the background task where our actual VPN plugin runs.

use windows::{
    self as Windows,
    core::*,
    ApplicationModel::Background::IBackgroundTaskInstance,
    ApplicationModel::Core::CoreApplication,
    Networking::Vpn::{IVpnPlugIn, VpnChannel},
    Win32::Foundation::{E_INVALIDARG, E_NOINTERFACE, E_UNEXPECTED, S_OK},
    Win32::System::WinRT::IActivationFactory,
};

/// The WinRT Activatable Class which acts as the entrypoint for the background tasks
/// which get invoked to handle the actual VPN tunnel.
#[implement(Windows::ApplicationModel::Background::IBackgroundTask)]
pub struct VpnBackgroundTask;

impl VpnBackgroundTask {
    fn Run(&self, task: &Option<IBackgroundTaskInstance>) -> Result<()> {
        let task = task.as_ref().ok_or(Error::from(E_UNEXPECTED))?;

        // Grab existing plugin instance from in-memory app properties or create a new one
        let app_props = CoreApplication::Properties()?;
        let plugin = if app_props.HasKey("plugin")? {
            app_props.Lookup("props")?.cast()?
        } else {
            let plugin: IVpnPlugIn = super::plugin::VpnPlugin.into();
            app_props.Insert("plugin", plugin.clone())?;
            plugin
        };

        // Call into VPN platform with the plugin object
        VpnChannel::ProcessEventAsync(plugin, task.TriggerDetails()?)?;

        Ok(())
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
/// it knows to query us via the `DllGetActivationFactory` function we export to get some
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

    *factory = None;

    // Return the appropriate factory based on which class was requested
    if activatableClassId == "VpnBackgroundTask" {
        *factory = Some(VpnBackgroundTaskFactory.into());
    }

    // Since `activatableClassId` is an _In_ parameter, the caller is responsible
    // for freeing. But, the HSTRING wrapper from the windows crate has a `Drop`
    // impl which will attempt to free it once it goes out of scope. Thus, we simply
    // call `forget` to skip the drop call. Ideally windows-rs would also provide an
    // HStringReference wrapper type without such a drop to avoid this altogether.
    std::mem::forget(activatableClassId);

    if factory.is_null() {
        E_NOINTERFACE
    } else {
        S_OK
    }
}