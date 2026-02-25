import MetalKit
import simd

/// Matches the Metal Uniforms struct — must stay in sync with ShaderSource
struct Uniforms {
    var mvp: simd_float4x4 = matrix_identity_float4x4
    var time: Float = 0
    var audioLevel: Float = 0
    var powerLevel: Float = 0
    var intensity: Float = 0
    var hudOpacity: Float = 1
    var scanlineIntensity: Float = 0.15
    var vignetteIntensity: Float = 1.2
    var screenHeight: Float = 1080
    var aspectRatio: Float = 16.0 / 9.0
    var orbCenterX: Float = 0       // screen-space offset from center
    var orbCenterY: Float = 0       // screen-space offset from center
    var orbScale: Float = 1.0       // 1.0 = full size
    var bgOpacity: Float = 1.0      // hex grid background opacity
    var bgAlpha: Float = 1.0        // overall background alpha (0 = transparent window)
    var noiseSeedX: Float = 0
    var noiseSeedY: Float = 0
    var rotationY: Float = 0
}

class Renderer: NSObject, MTKViewDelegate {
    let device: MTLDevice
    let commandQueue: MTLCommandQueue
    let hudRenderer: HUDTextRenderer
    var uniforms = Uniforms()
    var timeline: Timeline?

    // Pipeline states for each render pass
    let spherePipeline: MTLRenderPipelineState
    let blurHPipeline: MTLRenderPipelineState
    let blurVPipeline: MTLRenderPipelineState
    let compositePipeline: MTLRenderPipelineState

    // Sphere mesh
    let vertexBuffer: MTLBuffer
    let numVertices: Int
    var rotation: Float = 0

    // Offscreen textures for multi-pass bloom
    var texMain: MTLTexture?    // sphere render
    var texBlurH: MTLTexture?   // horizontal blur
    var texBlurV: MTLTexture?   // vertical blur (final bloom)
    var texW = 0
    var texH = 0

    init(device: MTLDevice, metalView: MTKView, hudRenderer: HUDTextRenderer) {
        self.device = device
        self.commandQueue = device.makeCommandQueue()!
        self.hudRenderer = hudRenderer

        // Generate sphere mesh (48 lat x 64 lon, matches vibetotext)
        let (meshData, vertCount) = Renderer.generateSphereMesh(nLat: 48, nLon: 64)
        self.numVertices = vertCount
        self.vertexBuffer = device.makeBuffer(bytes: meshData, length: meshData.count, options: .storageModeShared)!

        // Compile shaders from source at runtime
        let library: MTLLibrary
        do {
            library = try device.makeLibrary(source: ShaderSource.source, options: nil)
        } catch {
            fatalError("[Jarvis] Shader compilation failed: \(error)")
        }

        guard let quadVertexFn = library.makeFunction(name: "vertexShader") else {
            fatalError("[Jarvis] Could not find vertexShader")
        }

        // Vertex descriptor for sphere mesh: position(3f) + normal(3f) + bary(3f) = 36 bytes
        let vd = MTLVertexDescriptor()
        vd.attributes[0].format = .float3; vd.attributes[0].offset = 0; vd.attributes[0].bufferIndex = 0
        vd.attributes[1].format = .float3; vd.attributes[1].offset = 12; vd.attributes[1].bufferIndex = 0
        vd.attributes[2].format = .float3; vd.attributes[2].offset = 24; vd.attributes[2].bufferIndex = 0
        vd.layouts[0].stride = 36

        // Pass 1: Sphere mesh → RGBA16Float offscreen texture
        do {
            let desc = MTLRenderPipelineDescriptor()
            desc.vertexFunction = library.makeFunction(name: "vertex_sphere")
            desc.fragmentFunction = library.makeFunction(name: "fragment_sphere")
            desc.vertexDescriptor = vd
            desc.colorAttachments[0].pixelFormat = .rgba16Float
            desc.colorAttachments[0].isBlendingEnabled = true
            desc.colorAttachments[0].sourceRGBBlendFactor = .one
            desc.colorAttachments[0].destinationRGBBlendFactor = .oneMinusSourceAlpha
            desc.colorAttachments[0].sourceAlphaBlendFactor = .one
            desc.colorAttachments[0].destinationAlphaBlendFactor = .oneMinusSourceAlpha
            spherePipeline = try device.makeRenderPipelineState(descriptor: desc)
        } catch {
            fatalError("[Jarvis] Sphere pipeline failed: \(error)")
        }

        // Pass 2: Horizontal blur
        do {
            let desc = MTLRenderPipelineDescriptor()
            desc.vertexFunction = quadVertexFn
            desc.fragmentFunction = library.makeFunction(name: "fragmentBlurH")
            desc.colorAttachments[0].pixelFormat = .rgba16Float
            blurHPipeline = try device.makeRenderPipelineState(descriptor: desc)
        } catch {
            fatalError("[Jarvis] BlurH pipeline failed: \(error)")
        }

        // Pass 3: Vertical blur
        do {
            let desc = MTLRenderPipelineDescriptor()
            desc.vertexFunction = quadVertexFn
            desc.fragmentFunction = library.makeFunction(name: "fragmentBlurV")
            desc.colorAttachments[0].pixelFormat = .rgba16Float
            blurVPipeline = try device.makeRenderPipelineState(descriptor: desc)
        } catch {
            fatalError("[Jarvis] BlurV pipeline failed: \(error)")
        }

        // Pass 4: Composite → screen
        do {
            let desc = MTLRenderPipelineDescriptor()
            desc.vertexFunction = quadVertexFn
            desc.fragmentFunction = library.makeFunction(name: "fragmentComposite")
            desc.colorAttachments[0].pixelFormat = metalView.colorPixelFormat
            compositePipeline = try device.makeRenderPipelineState(descriptor: desc)
        } catch {
            fatalError("[Jarvis] Composite pipeline failed: \(error)")
        }

        super.init()
        metalView.delegate = self
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {
        uniforms.screenHeight = Float(size.height)
        uniforms.aspectRatio = Float(size.width / size.height)
    }

    /// Create or resize offscreen textures for bloom passes
    private func ensureTextures(w: Int, h: Int) {
        guard w != texW || h != texH || texMain == nil else { return }
        texW = w; texH = h

        let desc = MTLTextureDescriptor.texture2DDescriptor(
            pixelFormat: .rgba16Float, width: w, height: h, mipmapped: false
        )
        desc.usage = [.renderTarget, .shaderRead]
        desc.storageMode = .private

        texMain = device.makeTexture(descriptor: desc)
        texBlurH = device.makeTexture(descriptor: desc)
        texBlurV = device.makeTexture(descriptor: desc)
    }

    func draw(in view: MTKView) {
        // Update timeline (drives all animation)
        timeline?.update(uniforms: &uniforms)

        // Build MVP matrix for sphere mesh
        rotation += 0.006
        let proj = Renderer.perspective(fov: .pi / 4, aspect: uniforms.aspectRatio, near: 0.1, far: 100)
        let view_ = Renderer.translate(0, 0, -3.8)
        let scl = Renderer.scale(uniforms.orbScale)
        let model = scl * Renderer.rotateX(0.4) * Renderer.rotateY(rotation)
        uniforms.mvp = proj * view_ * model
        uniforms.rotationY = rotation

        guard let drawable = view.currentDrawable else { return }
        let w = drawable.texture.width
        let h = drawable.texture.height
        ensureTextures(w: w, h: h)

        guard let texMain, let texBlurH, let texBlurV,
              let cmdBuf = commandQueue.makeCommandBuffer() else { return }

        let clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 0)

        // ── Pass 1: Sphere mesh → texMain ──
        let rpd1 = MTLRenderPassDescriptor()
        rpd1.colorAttachments[0].texture = texMain
        rpd1.colorAttachments[0].loadAction = .clear
        rpd1.colorAttachments[0].clearColor = clearColor
        rpd1.colorAttachments[0].storeAction = .store

        if let enc = cmdBuf.makeRenderCommandEncoder(descriptor: rpd1) {
            enc.setRenderPipelineState(spherePipeline)
            enc.setVertexBuffer(vertexBuffer, offset: 0, index: 0)
            enc.setVertexBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 1)
            enc.setFragmentBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 1)
            enc.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: numVertices)
            enc.endEncoding()
        }

        // ── Pass 2: Horizontal blur — texMain → texBlurH ──
        let rpd2 = MTLRenderPassDescriptor()
        rpd2.colorAttachments[0].texture = texBlurH
        rpd2.colorAttachments[0].loadAction = .clear
        rpd2.colorAttachments[0].clearColor = clearColor
        rpd2.colorAttachments[0].storeAction = .store

        if let enc = cmdBuf.makeRenderCommandEncoder(descriptor: rpd2) {
            enc.setRenderPipelineState(blurHPipeline)
            enc.setFragmentTexture(texMain, index: 0)
            enc.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
            enc.endEncoding()
        }

        // ── Pass 3: Vertical blur — texBlurH → texBlurV ──
        let rpd3 = MTLRenderPassDescriptor()
        rpd3.colorAttachments[0].texture = texBlurV
        rpd3.colorAttachments[0].loadAction = .clear
        rpd3.colorAttachments[0].clearColor = clearColor
        rpd3.colorAttachments[0].storeAction = .store

        if let enc = cmdBuf.makeRenderCommandEncoder(descriptor: rpd3) {
            enc.setRenderPipelineState(blurVPipeline)
            enc.setFragmentTexture(texBlurH, index: 0)
            enc.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
            enc.endEncoding()
        }

        // ── Pass 4: Composite — texMain + texBlurV + HUD → screen ──
        guard let passDesc = view.currentRenderPassDescriptor else {
            cmdBuf.commit()
            return
        }

        if let enc = cmdBuf.makeRenderCommandEncoder(descriptor: passDesc) {
            enc.setRenderPipelineState(compositePipeline)
            enc.setFragmentBytes(&uniforms, length: MemoryLayout<Uniforms>.stride, index: 0)
            enc.setFragmentTexture(texMain, index: 0)               // sphere
            enc.setFragmentTexture(texBlurV, index: 1)              // bloom
            enc.setFragmentTexture(hudRenderer.texture, index: 2)   // HUD
            enc.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
            enc.endEncoding()
        }

        cmdBuf.present(drawable)
        cmdBuf.commit()
    }

    // MARK: - Sphere mesh generation (matches vibetotext)

    static func generateSphereMesh(nLat: Int, nLon: Int) -> ([UInt8], Int) {
        var data = [Float]()
        var numVerts = 0
        let bary: [[Float]] = [[1,0,0], [0,1,0], [0,0,1]]

        var grid = [[(Float, Float, Float)]]()
        for i in 0...nLat {
            let phi = Float.pi * Float(i) / Float(nLat)
            var row = [(Float, Float, Float)]()
            for j in 0...nLon {
                let theta = 2.0 * Float.pi * Float(j) / Float(nLon)
                let x = sin(phi) * cos(theta)
                let y = cos(phi)
                let z = sin(phi) * sin(theta)
                row.append((x, y, z))
            }
            grid.append(row)
        }

        for i in 0..<nLat {
            for j in 0..<nLon {
                let p00 = grid[i][j], p10 = grid[i][j+1], p01 = grid[i+1][j], p11 = grid[i+1][j+1]
                for (k, p) in [p00, p10, p01].enumerated() {
                    let b = bary[k]
                    data.append(contentsOf: [p.0, p.1, p.2, p.0, p.1, p.2, b[0], b[1], b[2]])
                }
                for (k, p) in [p10, p11, p01].enumerated() {
                    let b = bary[k]
                    data.append(contentsOf: [p.0, p.1, p.2, p.0, p.1, p.2, b[0], b[1], b[2]])
                }
                numVerts += 6
            }
        }

        let bytes = data.withUnsafeBufferPointer { ptr in
            Array(UnsafeBufferPointer(
                start: ptr.baseAddress!.withMemoryRebound(to: UInt8.self, capacity: data.count * 4) { $0 },
                count: data.count * 4
            ))
        }
        return (bytes, numVerts)
    }

    // MARK: - Matrix helpers

    static func perspective(fov: Float, aspect: Float, near: Float, far: Float) -> simd_float4x4 {
        let f = 1.0 / tan(fov / 2.0)
        let nf = near - far
        return simd_float4x4(columns: (
            SIMD4(f / aspect, 0, 0, 0),
            SIMD4(0, f, 0, 0),
            SIMD4(0, 0, (far + near) / nf, -1),
            SIMD4(0, 0, (2 * far * near) / nf, 0)
        ))
    }

    static func translate(_ x: Float, _ y: Float, _ z: Float) -> simd_float4x4 {
        var m = matrix_identity_float4x4
        m.columns.3 = SIMD4(x, y, z, 1)
        return m
    }

    static func rotateX(_ angle: Float) -> simd_float4x4 {
        let c = cos(angle), s = sin(angle)
        return simd_float4x4(columns: (
            SIMD4(1, 0, 0, 0), SIMD4(0, c, -s, 0), SIMD4(0, s, c, 0), SIMD4(0, 0, 0, 1)
        ))
    }

    static func rotateY(_ angle: Float) -> simd_float4x4 {
        let c = cos(angle), s = sin(angle)
        return simd_float4x4(columns: (
            SIMD4(c, 0, s, 0), SIMD4(0, 1, 0, 0), SIMD4(-s, 0, c, 0), SIMD4(0, 0, 0, 1)
        ))
    }

    static func scale(_ s: Float) -> simd_float4x4 {
        return simd_float4x4(columns: (
            SIMD4(s, 0, 0, 0), SIMD4(0, s, 0, 0), SIMD4(0, 0, s, 0), SIMD4(0, 0, 0, 1)
        ))
    }
}
