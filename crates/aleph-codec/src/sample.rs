pub(crate) fn index(x: usize, y: usize, c: usize, width: usize, components: usize) -> usize {
    (y * width + x) * components + c
}

/// Lossless-JPEG predictor selector 1 (Px = Ra) with ITU-T81 H.1.2.1 boundary
/// rules, applied per component. `samples` must hold every neighbor the rules
/// reference already reconstructed (left for Ra, above for Rb).
pub(crate) fn predict(
    samples: &[u16],
    x: usize,
    y: usize,
    c: usize,
    width: usize,
    components: usize,
    precision: u8,
) -> u16 {
    if y == 0 {
        if x == 0 {
            1u16 << (precision - 1)
        } else {
            samples[index(x - 1, y, c, width, components)]
        }
    } else if x == 0 {
        samples[index(x, y - 1, c, width, components)]
    } else {
        samples[index(x - 1, y, c, width, components)]
    }
}
