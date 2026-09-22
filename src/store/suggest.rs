// --- store::suggest ---
// "Did you mean" for command names. Hand-rolled because a dependency for
// fifteen lines of edit distance is not worth its place in the budget.

/// Up to three names close to `name`, nearest first.
pub(crate) fn suggest<'a>(name: &str, candidates: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut close: Vec<(&str, usize)> = candidates
        .map(|candidate| (candidate, distance(name, candidate)))
        .filter(|(_, distance)| *distance <= 2)
        .collect();
    close.sort_by_key(|(_, distance)| *distance);
    close.truncate(3);
    close.into_iter().map(|(candidate, _)| candidate).collect()
}

/// Levenshtein distance; names are ASCII by construction.
fn distance(a: &str, b: &str) -> usize {
    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, &left) in a.iter().enumerate() {
        let mut current = Vec::with_capacity(b.len() + 1);
        current.push(i + 1);
        for (j, &right) in b.iter().enumerate() {
            let substitute = previous[j] + usize::from(left != right);
            current.push((previous[j + 1] + 1).min(current[j] + 1).min(substitute));
        }
        previous = current;
    }
    previous[b.len()]
}
