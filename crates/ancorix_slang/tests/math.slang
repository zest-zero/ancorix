// Runs on the CPU: `slangc -target exe` builds the same module the GPU gets,
// so the maths below is the maths the shaders use. Anything depending on
// screen derivatives (`aa`, `coverage`) can't be checked here - it has no
// meaning without neighbouring pixels.

import ancorix;

static int failures = 0;

// The name has to be a literal: a `String` passed through a function prints
// as its object, not its characters, on the host target.
#define CHECK(cond, name)                       \
    if (cond) { printf("ok   " name "\n"); }    \
    else { printf("FAIL " name "\n"); failures += 1; }

#define NEAR(got, want, name) CHECK(abs((got) - (want)) < 0.001, name)

export __extern_cpp int main() {
    // shapes: negative inside, zero on the edge, positive outside
    NEAR(Circle(10.0).distance(float2(0.0, 0.0)), -10.0, "circle centre");
    NEAR(Circle(10.0).distance(float2(10.0, 0.0)), 0.0, "circle edge");
    NEAR(Circle(10.0).distance(float2(20.0, 0.0)), 10.0, "circle outside");

    NEAR(Box(float2(20.0, 10.0)).distance(float2(0.0, 0.0)), -5.0, "box centre");
    NEAR(Box(float2(20.0, 10.0)).distance(float2(10.0, 0.0)), 0.0, "box edge");

    NEAR(Segment(float2(0.0, 0.0), float2(10.0, 0.0), 4.0).distance(float2(5.0, 2.0)),
         0.0, "segment edge");

    // operations
    let donut = Circle(10.0).subtract(Circle(5.0));
    CHECK(donut.distance(float2(7.5, 0.0)) < 0.0, "donut ring is inside");
    CHECK(donut.distance(float2(0.0, 0.0)) > 0.0, "donut hole is outside");
    CHECK(donut.distance(float2(20.0, 0.0)) > 0.0, "donut outside is outside");

    let pair = Circle(5.0).union(Box(float2(4.0, 40.0)));
    CHECK(pair.distance(float2(0.0, 15.0)) < 0.0, "union covers the bar");
    CHECK(pair.distance(float2(4.0, 0.0)) < 0.0, "union covers the circle");

    CHECK(Circle(10.0).outline(2.0).distance(float2(10.0, 0.0)) < 0.0, "outline sits on the edge");
    CHECK(Circle(10.0).outline(2.0).distance(float2(0.0, 0.0)) > 0.0, "outline is hollow");

    NEAR(Circle(10.0).grow(5.0).distance(float2(15.0, 0.0)), 0.0, "grow moves the edge");

    // transforms
    NEAR(Circle(10.0).at(float2(30.0, 0.0)).distance(float2(30.0, 0.0)), -10.0, "at moves the shape");
    NEAR(Circle(10.0).at(float2(30.0, 0.0)).distance(float2(40.0, 0.0)), 0.0, "at moves the edge too");
    NEAR(Circle(10.0).rotate(1.0).distance(float2(0.0, 0.0)), -10.0, "rotating a circle changes nothing");

    // a 20x10 box turned a quarter turn is 10 wide and 20 tall
    let turned = Box(float2(20.0, 10.0)).rotate(1.5707963);
    NEAR(turned.distance(float2(0.0, 10.0)), 0.0, "rotate turns the box");
    CHECK(turned.distance(float2(9.0, 0.0)) > 0.0, "rotated box is thinner across");

    // scaling keeps the result a true distance, so the edge lands where the
    // shape grew to and not somewhere in between
    NEAR(Circle(10.0).scale(2.0).distance(float2(20.0, 0.0)), 0.0, "scale moves the edge");
    NEAR(Circle(10.0).scale(2.0).distance(float2(30.0, 0.0)), 10.0, "scale keeps distances true");

    // the pivot trick: move, turn, move back
    let pivoted = Circle(5.0).at(float2(-20.0, 0.0)).rotate(1.5707963).at(float2(20.0, 0.0));
    CHECK(pivoted.distance(float2(20.0, -20.0)) < 0.0, "composition rotates about a pivot");

    // cutting corners: a 100x100 box chamfered by 20 loses its corner but
    // keeps the middle of each side
    let plate = Box(float2(100.0, 100.0)).chamfer(float2(100.0, 100.0), 20.0);
    CHECK(plate.distance(float2(0.0, 0.0)) < 0.0, "chamfered plate has a middle");
    CHECK(plate.distance(float2(49.0, 0.0)) < 0.0, "chamfer keeps the side");
    CHECK(plate.distance(float2(45.0, 45.0)) > 0.0, "chamfer takes the corner");

    // a window cut through it
    let window = plate.subtract(Circle(20.0));
    CHECK(window.distance(float2(0.0, 0.0)) > 0.0, "cut opens a window");
    CHECK(window.distance(float2(35.0, 0.0)) < 0.0, "the plate around it stays");

    // patterns stay in 0..1 and repeat
    NEAR(checker(float2(0.0, 0.0), 10.0), checker(float2(20.0, 20.0), 10.0), "checker repeats");
    CHECK(checker(float2(0.0, 0.0), 10.0) != checker(float2(10.0, 0.0), 10.0),
          "checker alternates");

    // gradients run 0 -> 1 across the shape
    NEAR(linear_gradient(float2(0.0, 0.5), 0.0), 0.0, "gradient starts at zero");
    NEAR(linear_gradient(float2(1.0, 0.5), 0.0), 1.0, "gradient ends at one");
    NEAR(radial_gradient(float2(0.0, 0.0), float2(0.0, 0.0), 50.0), 0.0, "radial centre");
    NEAR(radial_gradient(float2(50.0, 0.0), float2(0.0, 0.0), 50.0), 1.0, "radial edge");

    // distortions leave a point alone when they have no strength
    NEAR(wave(float2(7.0, 3.0), 0.0, 0.1, 1.0).x, 7.0, "wave with no amount does nothing");
    NEAR(swirl(float2(7.0, 3.0), float2(0.0, 0.0), 0.0, 50.0).y, 3.0, "swirl with no strength");
    // ...and move it when they do
    CHECK(wave(float2(7.0, 3.0), 5.0, 0.5, 1.0).x != 7.0, "wave moves the point");

    // colour
    let cyan = rgb(0x22d3ee);
    NEAR(Color.cyan.g, 1.0, "named colour");
    NEAR(Color.clear.a, 0.0, "clear is transparent");
    NEAR(cyan.r, 34.0 / 255.0, "rgb red channel");
    NEAR(cyan.g, 211.0 / 255.0, "rgb green channel");
    NEAR(cyan.b, 238.0 / 255.0, "rgb blue channel");
    NEAR(rgba(0x22d3ee80).a, 128.0 / 255.0, "rgba alpha channel");

    NEAR(float4(1.0, 1.0, 1.0, 1.0).fade(0.25).a, 0.25, "fade scales alpha");
    NEAR(float4(0.0, 0.0, 0.0, 1.0).lighten(0.5).r, 0.5, "lighten towards white");
    NEAR(float4(1.0, 1.0, 1.0, 1.0).darken(1.0).r, 0.0, "darken to black");

    // over: opaque top wins, transparent top keeps the bottom
    let red = float4(1.0, 0.0, 0.0, 1.0);
    let blue = float4(0.0, 0.0, 1.0, 1.0);
    NEAR(over(red, blue).r, 1.0, "opaque top wins");
    NEAR(over(red.fade(0.0), blue).b, 1.0, "clear top keeps bottom");

    // sRGB round trip
    NEAR(to_srgb(to_linear(float3(0.5, 0.5, 0.5))).r, 0.5, "srgb round trip");

    // degrees read better than radians, and must convert exactly
    NEAR(deg(180.0), 3.14159265, "deg to radians");
    NEAR(deg(45.0), 0.78539816, "deg on a right angle's half");

    // the bezier solve: both ends pinned, and control points on the diagonal
    // give back a straight line
    NEAR(bezier(0.0, 0.42, 0.0, 0.58, 1.0), 0.0, "bezier starts at zero");
    NEAR(bezier(1.0, 0.42, 0.0, 0.58, 1.0), 1.0, "bezier ends at one");
    NEAR(bezier(0.5, 0.42, 0.0, 0.58, 1.0), 0.5, "ease-in-out is symmetric");
    NEAR(bezier(0.3, 1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0), 0.3, "diagonal is linear");
    NEAR(bezier(0.75, 1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0), 0.75, "still linear further along");
    // ease-in starts slower than linear, ease-out starts faster
    CHECK(bezier(0.25, 0.42, 0.0, 1.0, 1.0) < 0.25, "ease-in lags at the start");
    CHECK(bezier(0.25, 0.0, 0.0, 0.58, 1.0) > 0.25, "ease-out leads at the start");
    // and it stays inside 0..1 for well-behaved control points
    for (int i = 0; i <= 20; i++) {
        let v = bezier(float(i) / 20.0, 0.42, 0.0, 0.58, 1.0);
        if (v < -0.001 || v > 1.001) { failures += 1; break; }
    }
    CHECK(true, "bezier stays in range");

    // the cheap shapers
    NEAR(bias(0.5, 1.0), 0.5, "bias with k=1 is identity");
    CHECK(bias(0.5, 2.0) < 0.5, "bias above 1 pushes late");
    NEAR(gain(0.5, 2.0), 0.5, "gain keeps the midpoint");
    NEAR(gain(0.0, 3.0), 0.0, "gain keeps the start");
    NEAR(gain(1.0, 3.0), 1.0, "gain keeps the end");

    NEAR(ping_pong(0.0), 0.0, "ping_pong starts down");
    NEAR(ping_pong(1.0), 1.0, "ping_pong turns around");

    // noise stays in range and is repeatable
    CHECK(hash(float2(1.0, 2.0)) == hash(float2(1.0, 2.0)), "hash is repeatable");
    CHECK(hash(float2(1.0, 2.0)) != hash(float2(2.0, 1.0)), "hash separates coordinates");
    for (int i = 0; i < 64; i++) {
        let n = fbm(float2(float(i) * 0.37, float(i) * 0.71));
        if (n < 0.0 || n > 1.0) { failures += 1; break; }
    }
    CHECK(true, "fbm stays in 0..1");

    // voronoi: the nearest seed is never farther than the second-nearest,
    // and the winning cell is always one of the nine searched
    let v = voronoi(float2(3.3, -1.7));
    CHECK(v.f1 <= v.f2, "voronoi orders f1 before f2");
    CHECK(v.f1 >= 0.0, "voronoi distance is never negative");
    CHECK(abs(v.id.x - floor(3.3)) <= 1.0 && abs(v.id.y - floor(-1.7)) <= 1.0,
          "voronoi picks a neighbouring cell");
    let v2 = voronoi(float2(3.3, -1.7));
    CHECK(v.id.x == v2.id.x && v.id.y == v2.id.y, "voronoi is repeatable");
    CHECK(voronoi(float2(0.2, 0.2)).id.x != voronoi(float2(9.2, 9.2)).id.x,
          "voronoi separates distant points");

    printf(failures == 0 ? "\nall passed\n" : "\n%d FAILED\n", failures);
    return failures == 0 ? 0 : 1;
}
