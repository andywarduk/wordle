// Return current high resolution time
export function perf_now() {
    return performance.now();
}

// End a measured block
export function perf_end(measure_name, from, detail) {
    const opts = {
        start: from,
    };

    if (detail !== undefined) {
        opts.detail = detail;
    }

    // Measure elapsed
    return performance.measure(measure_name, opts);
}

// Write performance marker
export function perf_mark(marker) {
    // Write performance marker
    performance.mark(marker);
}
