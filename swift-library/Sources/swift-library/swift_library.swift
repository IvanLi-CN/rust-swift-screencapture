import AVFoundation
import CoreGraphics
import ScreenCaptureKit

//
//  ScreenCaptureKit-Recording-example
//
//  Created by Tom Lokhorst on 2023-01-18.
//

let stop_semaphore = DispatchSemaphore(value: 0)
let stopped_semaphore = DispatchSemaphore(value: 0)

func start_record(displayId: UInt32) {
  let semaphore = DispatchSemaphore(value: 0)

  Task {
    print("Equals: \(displayId == CGMainDisplayID())")
    await _start_record(displayId: CGDirectDisplayID(displayId))

    semaphore
      .signal()
  }
  semaphore.wait()

}

func stop_record() {
  print("Hit Return to end recording")

  stop_semaphore
    .signal()

  stopped_semaphore.wait()

}

func _start_record(displayId: CGDirectDisplayID) async {
  print("Starting screen recording of \(displayId) display")

  // Create a screen record ing
  do {
    // Check for screen recording permission, make sure your terminal has screen recording permission
    guard CGPreflightScreenCaptureAccess() else {
      throw RecordingError("No screen capture permission")
    }

    let url = URL(filePath: FileManager.default.currentDirectoryPath).appending(
      path: "recording \(Date()).mov")
    //    let cropRect = CGRect(x: 0, y: 0, width: 960, height: 540)
    let screenRecorder = try await ScreenRecorder(
      url: url, displayID: displayId, cropRect: nil)

    print("Starting screen recording of main display")
    try await screenRecorder.start()

    let dq: DispatchQueue = DispatchQueue(label: "stop")

    dq.async {
      stop_semaphore.wait()
      print("Stopping screen recording")

      Task {
        do {
          try await screenRecorder.stop()
          print("Recording ended, opening video")

          NSWorkspace.shared.open(url)
          stopped_semaphore
            .signal()

        } catch {
          print("Error during recording:", error)
        }

      }
    }

  } catch {
    print("Error during recording:", error)
  }

}

struct ScreenRecorder {
  private let videoSampleBufferQueue = DispatchQueue(label: "ScreenRecorder.VideoSampleBufferQueue")

  private let streamOutput: StreamOutput
  private var stream: SCStream

  private let displayID: CGDirectDisplayID

  init(url: URL, displayID: CGDirectDisplayID, cropRect: CGRect?) async throws {
    self.displayID = displayID

    // MARK: AVAssetWriter setup

    // Get size and pixel scale factor for display
    // Used to compute the highest possible qualitiy
    let displaySize = CGDisplayBounds(displayID).size

    // The number of physical pixels that represent a logic point on screen, currently 2 for MacBook Pro retina displays
    let displayScaleFactor: Int
    if let mode = CGDisplayCopyDisplayMode(displayID) {
      displayScaleFactor = mode.pixelWidth / mode.width
    } else {
      displayScaleFactor = 1
    }

    streamOutput = StreamOutput(displayID: self.displayID)

    // MARK: SCStream setup

    // Create a filter for the specified display
    let sharableContent = try await SCShareableContent.current
    guard let display = sharableContent.displays.first(where: { $0.displayID == displayID }) else {
      throw RecordingError("Can't find display with ID \(displayID) in sharable content")
    }
    let filter = SCContentFilter(display: display, excludingWindows: [])

    let configuration = SCStreamConfiguration()
    configuration.queueDepth = 6
    configuration.minimumFrameInterval = CMTime(value: 1, timescale: 30)
    configuration
      .pixelFormat = kCVPixelFormatType_32BGRA
    configuration.showsCursor = false

    // Make sure to take displayScaleFactor into account
    // otherwise, image is scaled up and gets blurry
    if let cropRect = cropRect {
      // ScreenCaptureKit uses top-left of screen as origin
      configuration.sourceRect = cropRect
      configuration.width = Int(cropRect.width) * displayScaleFactor
      configuration.height = Int(cropRect.height) * displayScaleFactor
    } else {
      configuration.width = Int(displaySize.width) * displayScaleFactor
      configuration.height = Int(displaySize.height) * displayScaleFactor
    }

    // Create SCStream and add local StreamOutput object to receive samples
    stream = SCStream(filter: filter, configuration: configuration, delegate: nil)
    try stream.addStreamOutput(
      streamOutput, type: .screen, sampleHandlerQueue: videoSampleBufferQueue)
  }

  func start() async throws {

    // Start capturing, wait for stream to start
    try await stream.startCapture()

    streamOutput.sessionStarted = true
  }

  func stop() async throws {
    try await stream.stopCapture()

  }

  private class StreamOutput: NSObject, SCStreamOutput {
    var sessionStarted = false
    var firstSampleTime: CMTime = .zero

    private let displayID: CGDirectDisplayID

    init(displayID: CGDirectDisplayID) {
      self.displayID = displayID

    }

    func stream(
      _ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer,
      of type: SCStreamOutputType
    ) {

      // Return early if session hasn't started yet
      guard sessionStarted else { return }

      // Return early if the sample buffer is invalid
      guard sampleBuffer.isValid else { return }

      // Retrieve the array of metadata attachments from the sample buffer
      guard
        let attachmentsArray = CMSampleBufferGetSampleAttachmentsArray(
          sampleBuffer, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
        let attachments = attachmentsArray.first
      else { return }

      // Validate the status of the frame. If it isn't `.complete`, return
      guard let statusRawValue = attachments[SCStreamFrameInfo.status] as? Int,
        let status = SCFrameStatus(rawValue: statusRawValue),
        status == .complete
      else { return }

      switch type {
      case .screen:
        let imageBuffer = CMSampleBufferGetImageBuffer(sampleBuffer)!

        // Lock the image buffer
        CVPixelBufferLockBaseAddress(imageBuffer, [])
        defer {
          CVPixelBufferUnlockBaseAddress(imageBuffer, [])
        }

        // Get the raw byte stream of the video data

        let width = CVPixelBufferGetWidth(imageBuffer)

        let height = CVPixelBufferGetHeight(imageBuffer)

        let bytesPerRow: Int = CVPixelBufferGetBytesPerRow(imageBuffer)
        let scaleFactor = CVPixelBufferGetBytesPerRowOfPlane(imageBuffer, 0) / bytesPerRow

        let baseAddress = CVPixelBufferGetBaseAddress(imageBuffer)
        let dataSize = bytesPerRow * height
        // let data = Data(bytes: baseAddress!, count: dataSize)

        // let bytes = [UInt8](data)

        // Get bytes UnsafeBufferPointer
        let bytesPointer = UnsafeBufferPointer(
          start: baseAddress?.assumingMemoryBound(to: UInt8.self), count: dataSize)

        frame(
          self.displayID,
          bytesPerRow,
          width,
          height,
          bytesPointer
        )

      case .audio:
        break

      @unknown default:
        break
      }
    }
  }
}

// AVAssetWriterInput supports maximum resolution of 4096x2304 for H.264
private func downsizedVideoSize(source: CGSize, scaleFactor: Int) -> (width: Int, height: Int) {
  let maxSize = CGSize(width: 4096, height: 2304)

  let w = source.width * Double(scaleFactor)
  let h = source.height * Double(scaleFactor)
  let r = max(w / maxSize.width, h / maxSize.height)

  return r > 1
    ? (width: Int(w / r), height: Int(h / r))
    : (width: Int(w), height: Int(h))
}

struct RecordingError: Error, CustomDebugStringConvertible {
  var debugDescription: String
  init(_ debugDescription: String) { self.debugDescription = debugDescription }
}
