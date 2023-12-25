pub(crate) enum DbusCommand {
    BluetoothDiscovering(String),
    GetAllBluetoothDevices,
    ConnectBluetoothDevice(String),
    DisconnectBluetoothDevice(String),
    PairBluetoothDevice(String),
    UnpairBluetoothDevice(String),
    TrustBluetoothDevice(String),
    UntrustBluetoothDevice(String),
    GetCurrentNetwork,
}