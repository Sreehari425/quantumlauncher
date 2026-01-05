Contains some **precompiled native libraries** for certain platforms.
These were *manually compiled from source by me*, because
they're really hard/inconvenient to build on your own.

The launcher will `include_bytes!()` these libraries
(embedded into the executable) for ease of distribution.

> Note: A basic description of the environment is provided
so you can reproduce the build on your own, but it's
unlikely that you'll get matching binary-perfect builds.

---

## `freebsd/liblwjgl64_x86_64.so`

- From [lwjgl 2.9.3](https://github.com/LWJGL/lwjgl/tree/70a8746f9aa1adaa440b61eb9f2d1b753d8a46f1)
- Purpose: LWJGL backend for FreeBSD.
- License: [LWJGL LICENSE](https://github.com/Mrmayman/quantumlauncher/tree/main/assets/licenses/LWJGL.txt)

### Compilation

- Built on **FreeBSD 13.4** using a **chroot environment**
  inside FreeBSD 14.3.
- Dependencies installed using `pkg`.
