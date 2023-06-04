public func frame(_ id: UInt32, _ bytes_per_row: Int, _ width: Int, _ height: Int, _ bytes: UnsafeBufferPointer<UInt8>) {
    __swift_bridge__$frame(id, bytes_per_row, width, height, bytes.toFfiSlice())
}
@_cdecl("__swift_bridge__$start_record")
func __swift_bridge__start_record (_ displayId: UInt32) {
    start_record(displayId: displayId)
}

@_cdecl("__swift_bridge__$stop_record")
func __swift_bridge__stop_record () {
    stop_record()
}



