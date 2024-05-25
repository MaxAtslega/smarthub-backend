pub(crate) enum SystemCommand {
    BluetoothDiscovering(String),
    GetAllBluetoothDevices,
    ConnectBluetoothDevice(String),
    DisconnectBluetoothDevice(String),
    PairBluetoothDevice(String),
    UnpairBluetoothDevice(String),
    TrustBluetoothDevice(String),
    UntrustBluetoothDevice(String),
    UpdateSystem,
    ListingSystemUpdates,
}