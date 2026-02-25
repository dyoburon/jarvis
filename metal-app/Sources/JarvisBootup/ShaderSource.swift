import Foundation

/// Metal shader source compiled at runtime via device.makeLibrary(source:)
enum ShaderSource {
    static let source = """
    #include <metal_stdlib>
    using namespace metal;

    // ── Quad vertex output (blur + composite passes) ──

    struct VertexOut {
        float4 position [[position]];
        float2 uv;
    };

    // ── Uniforms — must match Swift Uniforms struct exactly ──

    struct Uniforms {
        float4x4 mvp;
        float time;
        float audioLevel;
        float powerLevel;
        float intensity;
        float hudOpacity;
        float scanlineIntensity;
        float vignetteIntensity;
        float screenHeight;
        float aspectRatio;
        float orbCenterX;
        float orbCenterY;
        float orbScale;
        float bgOpacity;
        float bgAlpha;
        float noiseSeedX;
        float noiseSeedY;
        float rotationY;
    };

    // ── Sphere mesh vertex input/output ──

    struct SphereVIn {
        float3 position [[attribute(0)]];
        float3 normal   [[attribute(1)]];
        float3 bary     [[attribute(2)]];
    };

    struct SphereVOut {
        float4 position [[position]];
        float3 bary;
        float  ndotv;
        float3 worldPos;
        float  modelY;
    };

    // ── Fullscreen quad via vertex ID (blur + composite) ──

    vertex VertexOut vertexShader(uint vid [[vertex_id]]) {
        float2 pos[4] = { float2(-1,-1), float2(1,-1), float2(-1,1), float2(1,1) };
        float2 uv[4]  = { float2(0,1),   float2(1,1),  float2(0,0),  float2(1,0) };
        VertexOut out;
        out.position = float4(pos[vid], 0.0, 1.0);
        out.uv = uv[vid];
        return out;
    }

    // ══════════════════════════════════════════════════
    // NOISE — Simplex 3D (Stefan Gustavson)
    // ══════════════════════════════════════════════════

    float hash2d(float2 p) {
        float3 p3 = fract(float3(p.xyx) * 0.1031);
        p3 += dot(p3, p3.yzx + 33.33);
        return fract((p3.x + p3.y) * p3.z);
    }

    float3 mod289f3(float3 x) { return x - floor(x / 289.0) * 289.0; }
    float4 mod289f4(float4 x) { return x - floor(x / 289.0) * 289.0; }
    float4 permute4(float4 x) { return mod289f4((x * 34.0 + 1.0) * x); }
    float4 taylorInvSqrt4(float4 r) { return 1.79284291400159 - 0.85373472095314 * r; }

    float snoise(float3 v) {
        const float2 C = float2(1.0/6.0, 1.0/3.0);
        float3 i = floor(v + dot(v, float3(C.y)));
        float3 x0 = v - i + dot(i, float3(C.x));
        float3 g = step(x0.yzx, x0.xyz);
        float3 l = 1.0 - g;
        float3 i1 = min(g, l.zxy);
        float3 i2 = max(g, l.zxy);
        float3 x1 = x0 - i1 + C.x;
        float3 x2 = x0 - i2 + C.y;
        float3 x3 = x0 - 0.5;
        i = mod289f3(i);
        float4 p = permute4(permute4(permute4(
            i.z + float4(0, i1.z, i2.z, 1))
            + i.y + float4(0, i1.y, i2.y, 1))
            + i.x + float4(0, i1.x, i2.x, 1));
        float n_ = 0.142857142857;
        float3 ns = n_ * float3(2, 1, 0) - float3(1, 0.5, 0);
        float4 j = p - 49.0 * floor(p * ns.z * ns.z);
        float4 x_ = floor(j * ns.z);
        float4 y_ = floor(j - 7.0 * x_);
        float4 x2_ = x_ * ns.x + ns.y;
        float4 y2_ = y_ * ns.x + ns.y;
        float4 h = 1.0 - abs(x2_) - abs(y2_);
        float4 b0 = float4(x2_.xy, y2_.xy);
        float4 b1 = float4(x2_.zw, y2_.zw);
        float4 s0 = floor(b0) * 2.0 + 1.0;
        float4 s1 = floor(b1) * 2.0 + 1.0;
        float4 sh = -step(h, float4(0.0));
        float4 a0 = b0.xzyw + s0.xzyw * sh.xxyy;
        float4 a1 = b1.xzyw + s1.xzyw * sh.zzww;
        float3 p0 = float3(a0.xy, h.x);
        float3 p1 = float3(a0.zw, h.y);
        float3 p2 = float3(a1.xy, h.z);
        float3 p3 = float3(a1.zw, h.w);
        float4 norm = taylorInvSqrt4(float4(dot(p0,p0),dot(p1,p1),dot(p2,p2),dot(p3,p3)));
        p0 *= norm.x; p1 *= norm.y; p2 *= norm.z; p3 *= norm.w;
        float4 m = max(0.6 - float4(dot(x0,x0),dot(x1,x1),dot(x2,x2),dot(x3,x3)), 0.0);
        m = m * m;
        return 42.0 * dot(m * m, float4(dot(p0,x0),dot(p1,x1),dot(p2,x2),dot(p3,x3)));
    }

    // ══════════════════════════════════════════════════
    // HEX GRID BACKGROUND
    // ══════════════════════════════════════════════════

    float hexDist(float2 p) {
        p = abs(p);
        return max(dot(p, float2(0.8660254, 0.5)), p.y);
    }

    float4 hexCoords(float2 uv) {
        const float2 hs = float2(1.7320508, 1.0);
        float4 hc = floor(float4(uv, uv - float2(0.5 * hs.x, 0.5)) / float4(hs.x, hs.y, hs.x, hs.y)) + 0.5;
        float4 rg = float4(uv - hc.xy * hs, uv - (hc.zw + 0.5) * hs);
        return (dot(rg.xy, rg.xy) < dot(rg.zw, rg.zw))
            ? float4(rg.xy, hc.xy)
            : float4(rg.zw, hc.zw + 0.5);
    }

    float3 computeHexGrid(float2 screenUV, float aspectRatio, float time, float opacity) {
        float2 p = (screenUV - 0.5);
        p.x *= aspectRatio;

        float gridScale = 14.0;
        float4 h = hexCoords(p * gridScale);
        float2 localUV = h.xy;
        float2 hexID = h.zw;

        float d = 0.5 - hexDist(localUV);
        float edgeGlow = 0.008 / (d + 0.012);
        edgeGlow = clamp(edgeGlow, 0.0, 1.0);

        float cellRand = hash2d(hexID);
        float distFromCenter = length(hexID / gridScale);
        float wave = sin(distFromCenter * 4.0 - time * 1.5 + cellRand * 6.28) * 0.5 + 0.5;
        float flicker = smoothstep(0.92, 1.0, sin(time * 0.8 + cellRand * 6.28)) * 0.4;
        float cellFill = exp(-hexDist(localUV) * 8.0) * wave * 0.12;

        float brightness = edgeGlow * (0.25 + wave * 0.3 + flicker) + cellFill;
        float3 col = float3(0.12, 0.55, 0.85) * brightness;

        float2 vc = screenUV - 0.5;
        float vignette = 1.0 - dot(vc, vc) * 1.5;
        col *= saturate(vignette);

        return col * opacity;
    }

    // ══════════════════════════════════════════════════
    // PASS 1: SPHERE MESH — vertex + fragment
    // Ported from vibetotext sphere.metal
    // ══════════════════════════════════════════════════

    vertex SphereVOut vertex_sphere(SphereVIn in [[stage_in]],
                                    constant Uniforms& u [[buffer(1)]]) {
        // No amplitude-based expansion (sphere stays constant size)
        float scale = 1.0;
        float3 disp = in.position * scale;

        // Subtle tangential displacement — dots drift along the surface
        float displacement_amp = 0.00000005;
        float speed = 0.1;
        float noise_scale = 5.0;
        float3 p = in.position * noise_scale;
        float t = u.time * speed;

        float dx = snoise(p + float3(t, 0.0, 0.0));
        float dy = snoise(p + float3(0.0, t, 0.0));
        float dz = snoise(p + float3(0.0, 0.0, t));

        float3 tang_disp = float3(dx, dy, dz) * displacement_amp;

        // Project to tangential (perpendicular to normal)
        float3 normal = normalize(in.normal);
        tang_disp -= dot(tang_disp, normal) * normal;

        // Apply and renormalize to keep on sphere
        disp += tang_disp;
        disp = normalize(disp) * scale;

        float3 new_normal = normalize(disp);
        float3 viewDir = normalize(float3(0, 0, 1));

        // Model-space Y: apply rotateY then rotateX(0.4) to get view-aligned latitude
        float3 n = normalize(disp);
        float cosY = cos(u.rotationY), sinY = sin(u.rotationY);
        float rotZ = -n.x * sinY + n.z * cosY;
        float cosX = cos(0.4), sinX = sin(0.4);
        float mY = n.y * cosX - rotZ * sinX;

        SphereVOut out;
        out.position = u.mvp * float4(disp, 1.0);

        // Apply screen-space offset for orbCenter (post-projection)
        out.position.x += u.orbCenterX * 2.0 * out.position.w;
        out.position.y -= u.orbCenterY * 2.0 * out.position.w;

        out.bary = in.bary;
        out.worldPos = disp / scale;
        out.ndotv = abs(dot(new_normal, viewDir));
        out.modelY = mY;
        return out;
    }

    fragment float4 fragment_sphere(SphereVOut in [[stage_in]],
                                    constant Uniforms& u [[buffer(1)]]) {
        float3 p = in.worldPos;
        float t = u.time;
        float amp = u.audioLevel;

        // Fresnel rim — recompute from smooth sphere normal to avoid triangle faceting
        float3 smoothNormal = normalize(p);
        float3 viewDir = normalize(float3(0, 0, 1));
        float ndotv = abs(dot(smoothNormal, viewDir));
        float fresnel = pow(1.0 - ndotv, 3.0);

        // Screen-space Y for smooth scan lines
        float screenY = in.position.y;

        // Horizontal scan lines — screen-space for perfect smoothness
        float scanFreq = 4.0;
        float scanLines = pow(sin(screenY * scanFreq + t * 1.5) * 0.5 + 0.5, 8.0);

        // Use model-space Y for consistent latitude lines
        float sphereY = in.modelY;

        // Sweeping bar — moves up/down when idle, converges to equator when speaking
        float barSweep = sin(t * 0.8) * 0.9;
        float barCenter = mix(barSweep, -0.05, smoothstep(0.0, 0.3, amp));

        // 3 lines — all overlap at barCenter when idle, spread apart with voice
        float lineThick = 0.025;
        float lineGlow  = 0.06;
        float spread = amp * 0.35;

        float positions[3] = {
            barCenter,
            barCenter + spread,
            barCenter - spread
        };

        float lines = 0.0;
        for (int i = 0; i < 3; i++) {
            float d = abs(sphereY - positions[i]);
            lines += smoothstep(lineThick, 0.0, d);
            lines += smoothstep(lineGlow, lineThick, d) * 0.25;
        }
        lines = min(lines, 1.0);
        lines *= (0.8 + amp * 0.5);

        // Flicker — screen-space for smoothness
        float flicker = 1.0 - amp * 0.3 * sin(t * 17.0 + screenY * 0.5);

        // Combine: rim + scan lines + equator lines
        float baseIntensity = fresnel * 0.324
                            + scanLines * 0.081
                            + 0.024;
        float lineIntensity = lines * 0.315;
        float intensity = (baseIntensity + lineIntensity) * flicker;

        // Color: gradient across sphere — blue to pink
        float3 blueCol = float3(0.096, 0.426, 0.658);
        float3 pinkCol = float3(0.554, 0.285, 0.401);
        float grad = p.x * 0.5 + 0.5;
        float3 sphereCol = mix(pinkCol, blueCol, grad);

        // Lines get a brighter white tint
        float lineFactor = clamp(lineIntensity * 2.0, 0.0, 1.0);
        float3 col = mix(sphereCol, float3(0.9, 0.92, 0.95), lineFactor);

        float a = clamp(intensity * u.powerLevel, 0.0, 1.0);
        return float4(col * a, a);
    }

    // ══════════════════════════════════════════════════
    // PASS 2 & 3: GAUSSIAN BLOOM
    // 5-tap separable blur
    // ══════════════════════════════════════════════════

    fragment float4 fragmentBlurH(VertexOut in [[stage_in]],
                                  texture2d<float> tex [[texture(0)]]) {
        constexpr sampler s(filter::linear, address::clamp_to_edge);
        float2 texel = 1.0 / float2(tex.get_width(), tex.get_height());
        float weights[5] = {0.227027, 0.194596, 0.121622, 0.054054, 0.016216};
        float4 result = tex.sample(s, in.uv) * weights[0];
        for (int i = 1; i < 5; i++) {
            result += tex.sample(s, in.uv + float2(texel.x * i, 0)) * weights[i];
            result += tex.sample(s, in.uv - float2(texel.x * i, 0)) * weights[i];
        }
        return result;
    }

    fragment float4 fragmentBlurV(VertexOut in [[stage_in]],
                                  texture2d<float> tex [[texture(0)]]) {
        constexpr sampler s(filter::linear, address::clamp_to_edge);
        float2 texel = 1.0 / float2(tex.get_width(), tex.get_height());
        float weights[5] = {0.227027, 0.194596, 0.121622, 0.054054, 0.016216};
        float4 result = tex.sample(s, in.uv) * weights[0];
        for (int i = 1; i < 5; i++) {
            result += tex.sample(s, in.uv + float2(0, texel.y * i)) * weights[i];
            result += tex.sample(s, in.uv - float2(0, texel.y * i)) * weights[i];
        }
        return result;
    }

    // ══════════════════════════════════════════════════
    // PASS 4: FINAL COMPOSITE
    // hex grid + dark circle + sphere + bloom + dot + HUD
    // ══════════════════════════════════════════════════

    fragment half4 fragmentComposite(VertexOut in [[stage_in]],
                                     texture2d<float> sphereTex [[texture(0)]],
                                     texture2d<float> bloomTex  [[texture(1)]],
                                     texture2d<half>  hudTex    [[texture(2)]],
                                     constant Uniforms& u [[buffer(0)]]) {
        constexpr sampler s(filter::linear, address::clamp_to_edge);

        // Sample sphere and bloom
        float4 main_c = sphereTex.sample(s, in.uv);
        float4 bloom_c = bloomTex.sample(s, in.uv);

        // Hex grid background
        float3 bg = computeHexGrid(in.uv, u.aspectRatio, u.time, u.bgOpacity);

        // Dark background circle behind sphere
        float2 p = (in.uv - 0.5);
        p.x *= u.aspectRatio;
        float2 center = float2(u.orbCenterX * u.aspectRatio, u.orbCenterY);
        float dist = length(p - center);
        float sphereRadius = 0.32 * u.orbScale;
        float bg_circle = 1.0 - smoothstep(sphereRadius * 1.0, sphereRadius * 1.15, dist);
        float bg_a = bg_circle * 0.88 * u.powerLevel;

        // Center dot
        float amp = clamp(u.audioLevel * 2.0, 0.0, 1.0);
        float coreSize = (0.06 + amp * 0.08) * u.orbScale;
        float glowSize = coreSize + (0.08 + amp * 0.06) * u.orbScale;
        float dot_core = 1.0 - smoothstep(0.0, coreSize, dist);
        float dot_glow = (1.0 - smoothstep(coreSize, glowSize, dist)) * 0.3;
        float dot_a = (dot_core + dot_glow) * u.powerLevel;
        float3 idle_col = float3(0.894, 0.894, 0.906);
        float3 voice_col = float3(0.220, 0.576, 0.906);
        float3 dot_col = mix(idle_col, voice_col, amp);

        // HUD text (flip Y)
        float2 hudUV = float2(in.uv.x, 1.0 - in.uv.y);
        float3 hudColor = float3(hudTex.sample(s, hudUV).rgb) * u.hudOpacity;

        // ── Composite layers ──

        // Start with hex grid
        float3 color = bg;

        // Dark circle dims hex grid where sphere sits
        color = mix(color, float3(0.0), bg_a);

        // Sphere on top (premultiplied alpha blend)
        color = color * (1.0 - main_c.a) + main_c.rgb;

        // Bloom additive
        color += bloom_c.rgb * 0.9;

        // Center dot additive
        color += dot_col * dot_a;

        // HUD additive
        color += hudColor;

        // ── Post-processing ──

        // CRT scan lines
        float scanline = sin(in.uv.y * u.screenHeight * 3.14159) * 0.5 + 0.5;
        scanline = pow(scanline, 0.8) * u.scanlineIntensity + (1.0 - u.scanlineIntensity);
        color *= scanline;

        // Vignette
        float2 vc = in.uv - 0.5;
        float vignette = 1.0 - dot(vc, vc) * u.vignetteIntensity;
        color *= saturate(vignette);

        // Subtle flicker
        color *= 1.0 + sin(u.time * 30.0) * 0.004;

        // Alpha for window transparency during collapse
        float contentBrightness = saturate(length(color) * 2.5);
        float alpha = u.bgAlpha + (1.0 - u.bgAlpha) * contentBrightness;

        return half4(half3(color), half(alpha));
    }
    """
}
