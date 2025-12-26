# Community Fixes

A collection of workarounds to issues, found by the community.
Each fix includes a description, workaround, and credits.

---

## Out of stack Error (Vulkan Mod)

### Description

Occurs when a Vulkan renderer mod (like VulkanMod) attempts to spawn too many threads.  
Each native thread reserves a portion of stack memory,
and on systems using **NVIDIA proprietary drivers**,
this default stack size is quite large (around **1 MB per thread**). 

When the mod creates a large number of threads,
the system quickly runs out of available native stack space,
leading to crashes or errors such as `out of stack space`
or `OutOfMemoryError: unable to create new native thread`

### Fix

Add the following to your Java arguments:

```java
-Dorg.lwjgl.system.stackSize=256
```

### Credits

Discovered by [Apical Shark](https://github.com/apicalshark/).

---

## LLVM 3.8 Invalid Record (WGPU Graphics Backend)

### Description

```
error: Invalid record (Producer: 'LLVM3.8.0' Reader: 'LLVM 3.8.0')
```

When wgpu (UI graphics backend) tries to compile or read cached shader data
using those backends, the underlying LLVM/DXIL parser fails, resulting in this error.

### Fix

Force QuantumLauncher to use the OpenGL backend instead of DirectX 12 / Vulkan.

Either set the environment variable (temporary): `WGPU_BACKEND=gl`

Or (recommended), create qldir.txt in `QuantumLauncher/` folder.
(see [FAQ](https://mrmayman.github.io/quantumlauncher/faq)), and input this in it:

```txt
.
i_opengl
```

### Credits

- Discovered by Spicy Bee (balos_sandor) at discord
- Workaround by [Aurlt](https://github.com/Aurlt)  

---


# Outdated

The following issues were fixed/have easier solutions in newer
launcher versions but may still be helpful to some.


---

## SSLHandshakeException with elyby/littleskin, on Windows (unable to find valid certification path)

### Description

When launching Minecraft with elyby/littleskin on Windows, authentication fails due to an **SSL handshake error**, due to unpatched Java.

The error goes like:

```txt
[authlib-injector] [ERROR] Failed to fetch metadata:
javax.net.ssl.SSLHandshakeException: 
sun.security.validator.ValidatorException: PKIX path 
building failed: 
sun.security.provider.certpath.SunCertPathBuilderException: 
unable to find valid certification path to requested target
```

### Fix

Either:

1) Update to QuantumLauncher 0.5.0
2) Override the Java version of the instance with a newer build of Java 8
3) Add the following Java argument: `--Djavax.net.ssl.trustStoreType=Windows-ROOT`

### Credits

Discovered by blackbananaman1 at discord

Workaround by [Sreehari425](https://github.com/Sreehari425/)

---

## Minecraft fails to launch (missing xrandr command on Linux)

### Description

Older versions of Minecraft have a hardcoded dependency on the **xrandr** command,
which is part of the **X11 display management utilities**.

If the xrandr command can't be found,
Minecraft may fail to launch without a clear error message.
This can happen both on X11 and on Wayland (when running through XWayland).

```java
java.lang.ExceptionInInitializerError ...
Caused by: java.lang.ArrayIndexOutOfBoundsException: 0
        at org.lwjgl.opengl.LinuxDisplay.getAvailableDisplayModes(LinuxDisplay.java:951)
        at org.lwjgl.opengl.LinuxDisplay.init(LinuxDisplay.java:738)
        at org.lwjgl.opengl.Display.<clinit>(Display.java:138)
```

### Fix

Install `xrandr` using your package manager. Eg:

```bash
# Fedora / RHEL-based systems
sudo dnf install xrandr

# Arch / Manjaro
sudo pacman -S xorg-xrandr

# Ubuntu / Debian
sudo apt install x11-xserver-utils
```

### Credits

Discovered and confirmed by [mrmayman](https://github.com/mrmayman).
