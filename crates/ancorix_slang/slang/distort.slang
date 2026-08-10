// Coordinate distortions. Each takes a point and returns a moved one, so
// they compose and can be handed to any shape or texture lookup:
//
//     let p = wave(s.local, 8.0, 0.05, s.time);
//     return s.color.fade(Circle(60.0).coverage(p));

import noise;

/// Pushes the point sideways along a sine, `amount` pixels at `frequency`
/// waves per pixel. `phase` drives it - pass `s.time` to make it move.
public float2 wave(float2 p, float amount, float frequency, float phase) {
    return p + float2(sin(p.y * frequency + phase) * amount, 0.0);
}

/// Rings spreading out from `centre`, like a drop on water.
public float2 ripple(float2 p, float2 centre, float amount, float wavelength, float phase) {
    let offset = p - centre;
    let distance = length(offset);
    if (distance < 0.0001) { return p; }

    const float TAU = 6.28318530718;
    let push = sin(distance / wavelength * TAU - phase) * amount;
    return p + normalize(offset) * push;
}

/// Twists the point around `centre`, harder near it and fading out by
/// `radius`.
public float2 swirl(float2 p, float2 centre, float strength, float radius) {
    let offset = p - centre;
    let fade = saturate(1.0 - length(offset) / radius);
    let angle = strength * fade * fade;

    let c = cos(angle);
    let s = sin(angle);
    return centre + float2(offset.x * c - offset.y * s, offset.x * s + offset.y * c);
}

/// Roughens the point with noise - the base of smoke, fire and flowing water.
public float2 turbulence(float2 p, float amount, float scale, float phase) {
    let x = fbm(p * scale + float2(phase, 0.0));
    let y = fbm(p * scale + float2(0.0, phase + 37.0));
    return p + (float2(x, y) - 0.5) * amount;
}
