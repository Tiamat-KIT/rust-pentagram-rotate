struct Uniforms {
    time: f32,
}

struct InstanceInput {
    @location(2) position: vec2<f32>,
    @location(3) scale: f32,
    @location(4) initialRotation: f32,
    @location(5) speed: vec2<f32>,
    @location(6) rotationSpeed: f32,
}

@binding(0) @group(0) var<uniform> uniforms: Uniforms;

@vertex
fn vertexMain(
    @location(0) position: vec2<f32>,
    @builtin(instance_index) instanceIdx: u32,
    instance: InstanceInput,
) -> @builtin(position) vec4<f32> {
    // 時間に基づいて回転と移動を計算
    let rotation = instance.initialRotation + uniforms.time * instance.rotationSpeed;
    let moveX = instance.position.x + instance.speed.x * uniforms.time;
    let moveY = instance.position.y + instance.speed.y * uniforms.time;

    // 回転行列
    let c = cos(rotation);
    let s = sin(rotation);
    let rotMatrix = mat2x2<f32>(
        c, -s,
        s, c
    );

    // スケーリングと回転を適用
    let scaledPos = position * instance.scale;
    let rotatedPos = rotMatrix * scaledPos;
    
    // 最終位置の計算（画面内でラップする）
    var wrappedX = select(moveX, moveX + 2.0, moveX < -1.0);
    wrappedX = select(wrappedX, wrappedX - 2.0, wrappedX > 1.0);
    var wrappedY = select(moveY, moveY + 2.0, moveY < -1.0);
    wrappedY = select(wrappedY, wrappedY - 2.0, wrappedY > 1.0);

    let finalPos = vec2<f32>(
        rotatedPos.x + wrappedX,
        rotatedPos.y + wrappedY
    );

    return vec4<f32>(finalPos, 0.0, 1.0);
}

@fragment
fn fragmentMain() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 0.0, 1.0); // 黄色で塗りつぶし
}