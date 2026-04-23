// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "JarvisBootup",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "JarvisBootup",
            path: "Sources/JarvisBootup"
        )
    ]
)
