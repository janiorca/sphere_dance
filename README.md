# sphere_dance
Using rust to create a real 4K intro

###For easier development
During development use following to enable loading shader from shader.glsl and have movable camera and logs
```
 xargo run --target i686-pc-windows-msvc --features logger
```

###For the release version

First compile release version 
```
 xargo rustc --release --target i686-pc-windows-msvc -- --emit=obj
``` 

Then use crinkler to compress
```
 ..\..\..\..\..\tools\crinkler /OUT:mini.exe /SUBSYSTEM:WINDOWS miniwin.o /ENTRY:mainCRTStartup "/LIBPATH:C:\Program Files (x86)\Windows Kits\10\Lib\10.0.18362.0\um\x86" gdi32.lib user32.lib opengl32.lib kernel32.lib
 ```