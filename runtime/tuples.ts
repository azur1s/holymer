export function eqTuples(a: any[], b: any[]) {
    if (a.length !== b.length) return false;
    return a.every((elem, i) => b[i] === elem);
}