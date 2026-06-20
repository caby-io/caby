# Third-party software

The Caby Docker image bundles the following system libraries to support image manipulation. Per-package copyright and license notices are included in the image at `/usr/share/doc/<package>/copyright` and satisfy attribution requirements for the LGPL-licensed components below.

| Library  | Upstream                               | License           |
| -------- | -------------------------------------- | ----------------- |
| libvips  | https://github.com/libvips/libvips     | LGPL-2.1-or-later |
| libheif  | https://github.com/strukturag/libheif  | LGPL-3.0-or-later |
| libde265 | https://github.com/strukturag/libde265 | LGPL-3.0-or-later |
| glib2    | https://gitlab.gnome.org/GNOME/glib    | LGPL-2.1-or-later |

To inspect the bundled copyright text inside a running container:

```
docker exec -it <container> ls /usr/share/doc | grep -E 'vips|heif|de265|glib'
docker exec -it <container> cat /usr/share/doc/libvips42/copyright
```

## HEVC patent notice

`libde265` is open-source software, but the HEVC codec it decodes includes patents.

> TODO: document how to produce a non-HEVC variant.
