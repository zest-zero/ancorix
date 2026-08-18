import ancorix;

float4 pixel(Surface s) {
    let ring = Circle(s.size.x * 0.4).subtract(Circle(s.size.x * 0.2));
    return ring.paint(s, s.color);
}
