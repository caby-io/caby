
export const join = (...paths: Array<string>): string => {
    return paths
        .filter((p) => p != '' && p != '/' && p != null)
        .map((p) => {
            while (p.charAt(0) === '/') {
                p = p.substring(1);
            }
            return p
        }).join('/');
};

export const parent = (path: string): string => {
    return path.substring(0, path.lastIndexOf('/'))
}
