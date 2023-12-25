pub(crate) enum DbusCommand {
    BluetoothDiscovering(String),
    GetAllConnectedDevices,
    GetAllPairedDevices,
    GetAllDevices,
    ConnectDevice(String),
    DisconnectDevice(String),
    PairDevice(String),
    UnpairDevice(String),
    TrustDevice(String),
    UntrustDevice(String),
}