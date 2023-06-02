// swift-tools-version: 5.8
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
  name: "swift-library",
  platforms: [.macOS(.v13)],
  products: [
    .library(name: "swift-library", type: .static, targets: ["swift-library"])
  ],
  dependencies: [],
  targets: [
    .target(
      name: "swift-library",
      dependencies: []
    )
  ]
)
