
const join = (...paths: Array<string>): string => {
    let joined = '';
    paths
        .filter((p) => p != '' && p != '/' && p != null)
        .forEach((p) => {
            while (p.charAt(0) === '/') {
                p = p.substring(1);
            }
            joined += `/${p}`;
        });
    return joined;
};

export { join };