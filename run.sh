xargo rustc --release --target i686-pc-windows-msvc -- --emit=obj
./tools/Crinkler/releases/crinkler23/Win64/Crinkler.exe /OUT:mini.exe /SUBSYSTEM:WINDOWS ./target/i686-pc-windows-msvc/release/deps/miniwin.o /ENTRY:mainCRTStartup "/LIBPATH:C:\Program Files (x86)\Windows Kits\10\Lib\10.0.18362.0\um\x86" gdi32.lib user32.lib opengl32.lib kernel32.lib winmm.lib
./mini.exe